use super::error::{Error, Result};
use super::huffman_tree::HuffmanTree;
use super::marker::Marker;
use super::util::{read_u16_be, read_u8};
use std::io;
use std::io::Read;

struct Image {
    frame_header: Option<FrameHeader>,
    scans: Vec<Scan>,
    quantization_tables: [Option<QuantizationTable>; 4],
    ac_huffman_tables: [Option<HuffmanTree>; 4],
    dc_huffman_tables: [Option<HuffmanTree>; 4],
}

struct FrameHeader {
    encoding_process: EncodingProcess,
    precision: u8,
    max_lines: u16,
    max_samples_per_line: u16,
    components_count: u8,
    component_headers: [Option<FrameComponentHeader>; 4],
}

struct FrameComponentHeader {
    id: u8,
    horizontal_sampling_factor: u8,
    vertical_sampling_factor: u8,
    quantization_table_selector: u8,
}

enum EncodingProcess {
    Unknown,
    BaselineDct,
    ExtendedSequentialDctHc,
    ProgressiveDctHc,
    LosslessHc,
    ExtendedSequentialDctAc,
    ProgressiveDctAc,
    LosslessAc,
}

type QuantizationTable = Vec<u8>;

struct Scan {
    scan_header: ScanHeader,
    // components: Vec<Component>,
    // mcus: Vec<Mcu>,
}

struct ScanHeader {
    components_count: u8,
    component_headers: [Option<ScanComponentHeader>; 4],
}

struct ScanComponentHeader {
    scan_component_selector: u8,
    dc_entropy_coding_table_selector: u8,
    ac_entropy_coding_table_selector: u8,
}

pub type HuffmanTable = [Vec<u8>; 16];

// struct Component {}

// struct Mcu {}

pub struct Decoder<R: Read> {
    reader: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(reader: R) -> Self {
        return Self { reader };
    }

    pub fn decode(&mut self) -> Result<Vec<u8>> {
        self.parse()?;
        Ok(Vec::new())
    }

    fn parse(&mut self) -> Result<()> {
        let mut image = Image {
            frame_header: None,
            scans: Vec::new(),
            quantization_tables: [None, None, None, None],
            dc_huffman_tables: [None, None, None, None],
            ac_huffman_tables: [None, None, None, None],
        };

        loop {
            let marker = Marker::from_reader(&mut self.reader);
            match marker {
                Ok(Marker::StartOfImage) => println!("Marker: Start of Image"),
                Ok(Marker::ApplicationSegment(n, size)) => {
                    println!("Marker: Application Default Header({}) - {}", n, size);
                    skip_bytes(&mut self.reader, size - 2)?;
                }
                Ok(Marker::Comment(size)) => {
                    println!("Marker: Comment - {}", size);
                    self.parse_comment(size)?;
                }
                Ok(Marker::DefineQuantizationTable(size)) => {
                    println!("Marker: Define Quantization Table - {}", size);
                    let tables = self.parse_quantization_table(size)?;
                    for table in tables {
                        image.quantization_tables[table.0 as usize] = Some(table.1);
                    }
                }
                Ok(Marker::StartOfFrame(n, size)) => {
                    println!("Marker: Start of Frame({}) - {}", n, size);
                    image.frame_header = Some(self.parse_frame_header(n, size)?);
                }
                Ok(Marker::DefineHuffmanTable(size)) => {
                    println!("Marker: Define Huffman Table - {}", size);
                    let table_infos = self.parse_huffman_table(size)?;
                    for table_info in table_infos {
                        let tree = HuffmanTree::new(&table_info.2);
                        tree.print_codes();
                        if table_info.0 == 0 {
                            image.dc_huffman_tables[table_info.1 as usize] = Some(tree);
                        } else {
                            image.ac_huffman_tables[table_info.1 as usize] = Some(tree);
                        }
                    }
                }
                Ok(Marker::StartOfScan(size)) => {
                    println!("Marker: Start of Scan - {}", size);
                    let scan_header =
                        self.parse_scan_header(size, &image.frame_header.as_ref().unwrap())?;
                    image.scans.push(Scan { scan_header });

                    self.decode_scan();
                }
                Ok(Marker::EndOfImage) => {
                    println!("Marker: End of Image");
                    break;
                }
                Err(_) => return Err(Error::Parse("Non allowed marker found")),
            }
        }
        Ok(())
    }

    fn parse_comment(&mut self, size: u16) -> Result<()> {
        let mut comment_raw = vec![0; (size as usize) - 2];
        self.reader.read_exact(&mut comment_raw)?;

        if let Ok(comment) = String::from_utf8(comment_raw.clone()) {
            println!("\t{}", comment);
        } else {
            println!("\t{:?}", comment_raw);
        }

        Ok(())
    }

    fn parse_huffman_table(&mut self, size: u16) -> Result<Vec<(u8, u8, HuffmanTable)>> {
        let mut bytes_read = 0;

        let mut tables = Vec::new();

        while bytes_read < (size - 2) {
            let table_info = read_u8(&mut self.reader)?;
            bytes_read += 1;
            let huffman_table_class = (table_info & 0xf0) >> 4; // 0 == DC, 1 == AC
            println!(
                "\tHuffman table class: {}",
                if huffman_table_class == 0 { "DC" } else { "AC" }
            );
            let huffman_table_destination_identifier = table_info & 0x0f;
            println!(
                "\tHuffman table destination identifier: {}",
                huffman_table_destination_identifier
            );

            let mut numbers_of_huffman_codes_of_length = [0; 16];
            self.reader
                .read_exact(&mut numbers_of_huffman_codes_of_length)?;
            bytes_read += 16;
            println!(
                "\tHuffman code lengths: {:?}",
                numbers_of_huffman_codes_of_length
            );

            let mut huffman_table: HuffmanTable = Default::default();

            for (i, numbers_of_huffman_codes) in
                numbers_of_huffman_codes_of_length.iter().enumerate()
            {
                if *numbers_of_huffman_codes == 0 {
                    continue;
                }

                let mut huffman_values = Vec::new();
                for _ in 0..*numbers_of_huffman_codes {
                    let value = read_u8(&mut self.reader)?;
                    bytes_read += 1;
                    huffman_values.push(value);
                }
                huffman_table[i] = huffman_values;
            }

            println!("\tHuffman table: {:?}", huffman_table);

            tables.push((
                huffman_table_class,
                huffman_table_destination_identifier,
                huffman_table,
            ));
        }

        Ok(tables)
    }

    fn parse_quantization_table(&mut self, size: u16) -> Result<Vec<(u8, QuantizationTable)>> {
        let mut bytes_read = 0;

        let mut tables = Vec::new();

        while bytes_read < size - 2 {
            let quantization_table_info = read_u8(&mut self.reader)?;
            bytes_read += 1;

            let quantization_table_element_precision = (quantization_table_info & 0xf0) >> 4;
            assert!(quantization_table_element_precision == 0);
            println!(
                "\tQuantization table element precision: {}",
                quantization_table_element_precision
            );
            let quantization_table_destination_identifier = quantization_table_info & 0x0f;
            println!(
                "\tQuantization table destination identifer: {}",
                quantization_table_destination_identifier
            );

            let mut quantization_table = vec![0; 64];
            self.reader.read_exact(&mut quantization_table)?;
            println!("\tQuantization table: {:?}", quantization_table);
            bytes_read += 64 * (quantization_table_element_precision as u16 + 1);

            tables.push((
                quantization_table_destination_identifier,
                quantization_table,
            ));
        }

        Ok(tables)
    }

    fn parse_scan_header(&mut self, _size: u16, _frame_header: &FrameHeader) -> Result<ScanHeader> {
        // B.2.3

        let components_count = read_u8(&mut self.reader)?;
        println!("\tComponents count: {}", components_count);
        assert!(0 < components_count && components_count < 4);

        let mut scan_header = ScanHeader {
            components_count,
            component_headers: [None, None, None, None],
        };

        for i in 0..components_count {
            let scan_component_selector = read_u8(&mut self.reader)?;
            println!("\t\tScan component selector: {}", scan_component_selector);

            let entropy_coding_table_selectors = read_u8(&mut self.reader)?;
            let dc_entropy_coding_table_selector = (entropy_coding_table_selectors & 0xf0) >> 4;
            println!(
                "\t\tDc entropy coding table selector: {}",
                dc_entropy_coding_table_selector
            );
            let ac_entropy_coding_table_selector = entropy_coding_table_selectors & 0x0f;
            println!(
                "\t\tAc entropy coding table selector: {}",
                ac_entropy_coding_table_selector
            );

            let scan_component_header = ScanComponentHeader {
                scan_component_selector,
                dc_entropy_coding_table_selector,
                ac_entropy_coding_table_selector,
            };

            // TODO: The scan components should be sorted after the frame header,
            // but we assume that they are already in the correct order which is ok
            scan_header.component_headers[i as usize] = Some(scan_component_header);
        }

        // Skip 3 bytes that are meaningless for BaselineDCT
        skip_bytes(&mut self.reader, 3)?;

        Ok(scan_header)
    }

    fn parse_frame_header(&mut self, n: u8, _size: u16) -> Result<FrameHeader> {
        // B.2.2

        let encoding_process = match n {
            0 => {
                println!("\tEncoding process: Baseline DCT");
                EncodingProcess::BaselineDct
            }
            1 => {
                println!("\tEncoding process: Extended sequential DCT, Huffman coding");
                EncodingProcess::ExtendedSequentialDctHc
            }
            2 => {
                println!("\tEncoding process: Progressive DCT, Huffman coding");
                EncodingProcess::ProgressiveDctHc
            }
            3 => {
                println!("\tEncoding process: Lossless (sequential), Huffman coding");
                EncodingProcess::LosslessHc
            }
            9 => {
                println!("\tEncoding process: Extended sequential DCT, arithmetic coding");
                EncodingProcess::ExtendedSequentialDctHc
            }
            10 => {
                println!("\tEncoding process: Progressive DCT, arithmetic coding");
                EncodingProcess::ProgressiveDctAc
            }
            11 => {
                println!("\tEncoding process: Lossless (sequential), arithmetic coding");
                EncodingProcess::LosslessAc
            }
            _ => {
                println!("\tUnknown encoding process: {}", n);
                EncodingProcess::Unknown
            }
        };

        let precision = read_u8(&mut self.reader)?;
        println!("\tPrecision: {}", precision);
        assert!(precision == 8);

        let max_lines = read_u16_be(&mut self.reader)?;
        println!("\tMax lines: {}", max_lines);
        assert!(max_lines != 0);

        let max_samples_per_line = read_u16_be(&mut self.reader)?;
        println!("\tMax samples per line: {}", max_samples_per_line);

        let components_count = read_u8(&mut self.reader)?;
        println!("\tComponents count: {}", components_count);
        assert!(components_count == 3);

        let mut frame_header = FrameHeader {
            encoding_process,
            precision,
            max_lines,
            max_samples_per_line,
            components_count,
            component_headers: [None, None, None, None],
        };

        for i in 0..components_count {
            let id = read_u8(&mut self.reader)?;
            println!("\t\tComponent id: {}", id);

            let sampling_factor = read_u8(&mut self.reader)?;
            let horizontal_sampling_factor = (sampling_factor & 0xf0) >> 4;
            let vertical_sampling_factor = sampling_factor & 0x0f;
            println!(
                "\t\tHorizontal sampling factor: {}",
                horizontal_sampling_factor
            );
            println!("\t\tVertical sampling factor: {}", vertical_sampling_factor);

            let quantization_table_selector = read_u8(&mut self.reader)?;
            println!(
                "\t\tQuantization table selector: {}",
                quantization_table_selector
            );

            let component_header = FrameComponentHeader {
                id,
                horizontal_sampling_factor,
                vertical_sampling_factor,
                quantization_table_selector,
            };
            frame_header.component_headers[i as usize] = Some(component_header);
        }

        Ok(frame_header)
    }

    fn decode_scan(&self) {}
}

fn skip_bytes<R: Read>(reader: &mut R, size: u16) -> Result<()> {
    let size = size as u64;
    let to_skip = &mut reader.by_ref().take(size);
    let copied = io::copy(to_skip, &mut io::sink())?;
    if copied < size {
        Err(Error::Io(io::ErrorKind::UnexpectedEof.into()))
    } else {
        Ok(())
    }
}
