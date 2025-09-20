use std::ptr;

use std::ffi::CStr;

use esp_idf_svc::sys::esp_sr;

pub fn log_heap() {
    unsafe {
        use esp_idf_svc::sys::{heap_caps_get_free_size, MALLOC_CAP_INTERNAL, MALLOC_CAP_SPIRAM};

        log::info!(
            "Free SPIRAM heap size: {}",
            heap_caps_get_free_size(MALLOC_CAP_SPIRAM)
        );
        log::info!(
            "Free INTERNAL heap size: {}",
            heap_caps_get_free_size(MALLOC_CAP_INTERNAL)
        );
    }
}

pub fn print_partitions() {
    unsafe {
        log::info!("esp_partition_find_first");
        // 开始查找：传 null 表示从头开始
        let mut iterator = esp_sr::esp_partition_find(
            esp_sr::esp_partition_type_t_ESP_PARTITION_TYPE_ANY,
            esp_sr::esp_partition_subtype_t_ESP_PARTITION_SUBTYPE_ANY,
            ptr::null(),
        );
        if iterator.is_null() {
            log::error!("Couldn't find any partitions!");
            return;
        }

        while !iterator.is_null() {
            let part = esp_sr::esp_partition_get(iterator);
            if !part.is_null() {
                // 获取分区信息
                let partition = *part; // 解引用 C struct
                let name_cstr = CStr::from_ptr(partition.label.as_ptr());
                let name = name_cstr.to_string_lossy();

                log::info!(
                    "Partition: name={}, type=0x{:X}, subtype=0x{:X}, offset=0x{:X}, size={} bytes",
                    name,
                    partition.type_ as u8,
                    partition.subtype as u8,
                    partition.address,
                    partition.size
                );
            }

            // 继续查找下一个
            iterator = esp_sr::esp_partition_next(iterator);
        }
        esp_sr::esp_partition_iterator_release(iterator);
    }
}
