use std::{env, fs, io, path::PathBuf};

const ICON_PATH: &str = "assets/app.ico";
const ICON: &[u8] = include_bytes!("assets/app.ico");
const ICON_GROUP_ID: u16 = 1;
const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
const RT_MANIFEST: u16 = 24;

const MANIFEST_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="0.0.1.8" processorArchitecture="*" name="UiShowcase" type="win32"/>
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed={ICON_PATH}");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }
    let resource =
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR")).join("app-resources.res");
    fs::write(&resource, build_resources())?;
    println!("cargo:rustc-link-arg-bin=UiShowcase={}", resource.display());
    Ok(())
}

fn build_resources() -> Vec<u8> {
    assert_eq!(&ICON[..4], &[0, 0, 1, 0], "expected an ICO file");
    let count = u16::from_le_bytes([ICON[4], ICON[5]]);
    assert_ne!(count, 0, "ICO file has no frames");
    let mut group = ICON[..6].to_vec();
    let mut resource = Vec::new();

    // Win32 .res 文件头
    append_resource(&mut resource, 0, 0, &[]);
    for id in 1..=count {
        let start = 6 + 16 * usize::from(id - 1);
        let entry = &ICON[start..start + 16];
        let length = u32::from_le_bytes(entry[8..12].try_into().unwrap()) as usize;
        let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
        append_resource(&mut resource, RT_ICON, id, &ICON[offset..offset + length]);

        group.extend_from_slice(&entry[..12]);
        group.extend_from_slice(&id.to_le_bytes());
    }
    append_resource(&mut resource, RT_GROUP_ICON, ICON_GROUP_ID, &group);
    append_resource(&mut resource, RT_MANIFEST, 1, MANIFEST_XML.as_bytes());
    resource
}

fn append_resource(output: &mut Vec<u8>, kind: u16, name: u16, data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_le_bytes());
    output.extend_from_slice(&32_u32.to_le_bytes());
    for ordinal in [kind, name] {
        output.extend_from_slice(&u16::MAX.to_le_bytes());
        output.extend_from_slice(&ordinal.to_le_bytes());
    }
    output.extend_from_slice(&[0; 16]);
    output.extend_from_slice(data);
    output.resize(output.len().next_multiple_of(4), 0);
}
