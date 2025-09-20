use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

fn main() {
    embuild::espidf::sysenv::output();

    let (manifest_dir, target_dir) = get_target_dir();
    let copy_srmodels_flag = copy_srmodels(target_dir.clone());

    if std::env::consts::OS == "windows" {
        return;
    }
    generate_merged_sh(manifest_dir, target_dir, copy_srmodels_flag).unwrap();
}

fn get_target_dir() -> (PathBuf, PathBuf) {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let target_triple = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();

    let target_dir = manifest_dir
        .join("target")
        .join(&target_triple)
        .join(&profile);
    eprintln!("manifest_dir: {:?}", manifest_dir);
    eprintln!("target_dir: {:?}", target_dir);

    (manifest_dir, target_dir)
}

// 如果 build 过程中有  srmodels.bin 生成，则将其拷贝到 对应的 target 目录下
fn copy_srmodels(target_dir: PathBuf) -> bool {
    let target_build_dir = target_dir.join("build");
    eprintln!("target_build_dir: {:?}", target_build_dir);

    // 查找名字以 esp-idf-sys- 开头的目录
    let srmodels_src = fs::read_dir(target_build_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find_map(|p| {
            if p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("esp-idf-sys-")
            {
                let candidate = p
                    .join("out")
                    .join("build")
                    .join("srmodels")
                    .join("srmodels.bin");
                eprintln!("esp-idf-sys srmodels path: {:?}", candidate);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
            None
        });

    if let Some(srmodels_src) = srmodels_src {
        let srmodels_dest = target_dir.join("srmodels.bin");
        fs::create_dir_all(srmodels_dest.parent().unwrap()).unwrap();
        fs::copy(&srmodels_src, &srmodels_dest).unwrap();
        eprintln!("esp-idf-sys srmodels dest: {:?}", srmodels_dest);

        true
    } else {
        false
    }
}

// 生成 merged.sh
fn generate_merged_sh(
    manifest_dir: PathBuf,
    target_dir: PathBuf,
    copy_srmodels_flag: bool,
) -> Result<(), std::io::Error> {
    let merged_sh_path = manifest_dir.join("merged.sh");
    let mut merged_sh_file = File::create(&merged_sh_path)?;
    let app_name = env::var("CARGO_PKG_NAME").unwrap();
    let app_path = target_dir.join(app_name);

    writeln!(merged_sh_file, "#!/bin/bash")?;
    writeln!(merged_sh_file, "set -e")?;

    writeln!(merged_sh_file, "")?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo "=== esptool.py elf2image ===""#)?;
    writeln!(
        merged_sh_file,
        "esptool.py --chip esp32s3 elf2image {} -o {}.bin",
        app_path.to_str().unwrap(),
        app_path.to_str().unwrap()
    )?;

    writeln!(merged_sh_file, "")?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo "=== esptool.py image_info ===""#)?;
    writeln!(
        merged_sh_file,
        "esptool.py image_info {}.bin",
        app_path.to_str().unwrap()
    )?;
    writeln!(merged_sh_file, "")?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo "=== gen_esp32part.py ===""#)?;
    let partition_table_bin_path = target_dir.join("partition-table.bin");
    writeln!(
        merged_sh_file,
        "gen_esp32part.py {}",
        partition_table_bin_path.to_str().unwrap()
    )?;

    writeln!(merged_sh_file, "")?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(
        merged_sh_file,
        r#"echo "=== parse partition-table info ===""#
    )?;
    writeln!(merged_sh_file, "declare -A partitions_map")?;
    writeln!(merged_sh_file, "while IFS= read -r line; do")?;
    writeln!(
        merged_sh_file,
        "    if [[ $line =~ ^(nvs|phy_init|factory|model|voice_data), ]]; then"
    )?;
    writeln!(
        merged_sh_file,
        r#"        IFS=',' read -ra parts <<< "$line""#
    )?;
    writeln!(merged_sh_file, r#"        key="${{parts[0]}}""#)?;
    writeln!(merged_sh_file, r#"        value="${{parts[3]}}""#)?;
    writeln!(merged_sh_file, r#"        partitions_map["$key"]="$value""#)?;
    writeln!(merged_sh_file, "    fi")?;
    writeln!(
        merged_sh_file,
        "done < <(gen_esp32part.py {})",
        partition_table_bin_path.to_str().unwrap()
    )?;
    writeln!(merged_sh_file, r#"echo "parse partitions_map result:""#)?;
    writeln!(
        merged_sh_file,
        r#"for key in "${{!partitions_map[@]}}"; do"#
    )?;
    writeln!(
        merged_sh_file,
        r#"    echo "key: $key, value: ${{partitions_map[$key]}}""#
    )?;
    writeln!(merged_sh_file, "done")?;

    writeln!(merged_sh_file, "")?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo "=== esptool.py merge_bin ===""#)?;
    writeln!(
        merged_sh_file,
        "esptool.py --chip esp32s3 merge_bin --output merged.bin \\"
    )?;
    let bootloader_bin_path = target_dir.join("bootloader.bin");
    writeln!(
        merged_sh_file,
        "     0x0 {} \\",
        bootloader_bin_path.to_str().unwrap()
    )?;
    writeln!(
        merged_sh_file,
        "     0x8000 {} \\",
        partition_table_bin_path.to_str().unwrap()
    )?;
    if copy_srmodels_flag {
        writeln!(
            merged_sh_file,
            r#"     ${{partitions_map["factory"]}} {}.bin \"#,
            app_path.to_str().unwrap(),
        )?;
        let srmodels_bin_path = target_dir.join("srmodels.bin");
        writeln!(
            merged_sh_file,
            r#"     ${{partitions_map["model"]}} {}"#,
            srmodels_bin_path.to_str().unwrap()
        )?;
    } else {
        writeln!(
            merged_sh_file,
            r#"     ${{partitions_map["factory"]}} {}.bin"#,
            app_path.to_str().unwrap(),
        )?;
    }

    writeln!(merged_sh_file, "")?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(merged_sh_file, r#"echo """#)?;
    writeln!(
        merged_sh_file,
        r#"echo "=== esptool.py image_info merged.bin ===""#
    )?;
    writeln!(merged_sh_file, "esptool.py image_info merged.bin")?;
    Ok(())
}
