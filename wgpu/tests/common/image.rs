use std::{
    ffi::{OsStr, OsString},
    fs::File,
    io::{BufWriter, Cursor},
    path::Path,
    str::FromStr,
};

fn read_png(path: impl AsRef<Path>, width: u32, height: u32) -> Option<Vec<u8>> {
    let data = match std::fs::read(&path) {
        Ok(f) => f,
        Err(e) => {
            log::warn!(
                "image comparison invalid: file io error when comparing {}: {}",
                path.as_ref().display(),
                e
            );
            return None;
        }
    };
    let decoder = png::Decoder::new(Cursor::new(data));
    let (info, mut reader) = decoder.read_info().ok()?;
    if info.width != width {
        log::warn!("image comparison invalid: size mismatch");
        return None;
    }
    if info.height != height {
        log::warn!("image comparison invalid: size mismatch");
        return None;
    }
    if info.color_type != png::ColorType::RGBA {
        log::warn!("image comparison invalid: color type mismatch");
        return None;
    }
    if info.bit_depth != png::BitDepth::Eight {
        log::warn!("image comparison invalid: bit depth mismatch");
        return None;
    }

    let mut buffer = vec![0; info.buffer_size()];
    reader.next_frame(&mut buffer).ok()?;

    Some(buffer)
}

fn write_png(path: impl AsRef<Path>, width: u32, height: u32, data: &[u8]) {
    let file = BufWriter::new(File::create(path).unwrap());

    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Best);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&data).unwrap();
}

fn calc_difference(lhs: u8, rhs: u8) -> u8 {
    (lhs as i16 - rhs as i16).abs() as u8
}

pub fn compare_image_output(
    path: impl AsRef<Path> + AsRef<OsStr>,
    width: u32,
    height: u32,
    data: &[u8],
    tollerance: u8,
    max_outliers: usize,
) {
    let comparison_data = read_png(&path, width, height);

    if let Some(cmp) = comparison_data {
        assert_eq!(cmp.len(), data.len());

        let difference_data: Vec<_> = cmp
            .chunks_exact(4)
            .zip(data.chunks_exact(4))
            .flat_map(|(cmp_chunk, data_chunk)| {
                [
                    calc_difference(cmp_chunk[0], data_chunk[0]),
                    calc_difference(cmp_chunk[1], data_chunk[1]),
                    calc_difference(cmp_chunk[2], data_chunk[2]),
                    255,
                ]
            })
            .collect();

        let outliers: usize = difference_data
            .chunks_exact(4)
            .map(|colors| {
                (colors[0] > tollerance) as usize
                    + (colors[1] > tollerance) as usize
                    + (colors[2] > tollerance) as usize
            })
            .sum();

        let max_difference = difference_data
            .chunks_exact(4)
            .map(|colors| colors[0].max(colors[1]).max(colors[2]))
            .max()
            .unwrap();

        if outliers > max_outliers {
            // Because the deta is mismatched, lets output the difference to a file.
            let old_path = Path::new(&path);
            let difference_path = Path::new(&path).with_file_name(
                OsString::from_str(
                    &(old_path.file_stem().unwrap().to_string_lossy() + "-difference.png"),
                )
                .unwrap(),
            );

            write_png(&difference_path, width, height, &difference_data);

            panic!("Image data mismatch! Outlier count {} over limit {}. Max difference {}", outliers, max_outliers, max_difference)
        } else {
            println!(
                "{} outliers over max difference {}",
                outliers, max_difference
            );
        }
    } else {
        write_png(&path, width, height, data);
    }
}
