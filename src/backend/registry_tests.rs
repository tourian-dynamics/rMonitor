use super::*;

#[test]
fn test_registry_predef() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    assert_eq!(hkcu.hkey, HKEY_CURRENT_USER);
    assert!(!hkcu.owned);
}

#[test]
fn test_registry_read_write_delete() {
    #[cfg(target_os = "windows")]
    {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = r"Software\apps\Diagnostics\TestTemp";
        let (subkey, _) = hkcu.create_subkey(path).expect("Failed to create subkey");
        assert!(subkey.hkey != 0);
        assert!(subkey.owned);

        // Test string value
        let test_key = "TestString";
        let test_val = "Hello FFI Registry";
        subkey.set_value(test_key, &test_val.to_string()).expect("Failed to set string value");
        let read_val: String = subkey.get_value(test_key).expect("Failed to get string value");
        assert_eq!(read_val, test_val);

        // Test u32 value
        let test_key_u32 = "TestU32";
        let test_val_u32 = 42u32;
        subkey.set_value(test_key_u32, &test_val_u32).expect("Failed to set u32 value");
        let read_val_u32: u32 = subkey.get_value(test_key_u32).expect("Failed to get u32 value");
        assert_eq!(read_val_u32, test_val_u32);

        // Test deletion
        subkey.delete_value(test_key).expect("Failed to delete value");
        assert!(subkey.get_value::<String, _>(test_key).is_err());
    }
}

#[test]
fn test_free_functions_emulated() {
    let temp_file = std::env::temp_dir().join(format!("test_free_functions_emulated_{}.conf", std::process::id()));
    set_test_path_override(Some(temp_file.clone()));

    let hive = HKEY_CURRENT_USER;
    let path = "Software\\apps\\Diagnostics\\TestTempFree";
    let key = "TestFreeKey";
    let val = "Hello Free Emulated";
    
    // Write
    write_string(hive, path, key, val).expect("Failed write_string");
    
    // Read
    let read = read_string(hive, path, key).expect("Failed read_string");
    assert_eq!(read, val);
    
    // Delete/Clean up
    delete_value(hive, path, key).expect("Failed delete_value");
    
    // Verify deleted
    assert!(read_string(hive, path, key).is_none());

    set_test_path_override(None);
    let _ = std::fs::remove_file(temp_file);
}
