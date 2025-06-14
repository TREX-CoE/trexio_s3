use tokio::runtime;
use aws_sdk_s3::Client;
use libc::c_char;
use std::ffi::CStr;
use std::slice;


/// This function enables the use of Tokio in a library.
/// Found on Reddit:
/// https://www.reddit.com/r/rust/comments/18xrp8m/comment/kgj5uhc/
fn rt() -> Result<&'static runtime::Runtime, std::io::Error> {
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use thread_local::ThreadLocal;

    static RT: Lazy<ThreadLocal<OnceCell<runtime::Runtime>>> = Lazy::new(ThreadLocal::new);
    RT.get_or(OnceCell::new).get_or_try_init(|| {
        runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
    })
}


/// Data structure that contains the name of the bucket and the key
pub struct BucketAndKey {
    bucket: String,
    key: String,
}

impl BucketAndKey {

    /// Creates a BucketAndKey from a string.
    /// For example: /bucket/path/of/key -> "bucket" and
    /// "path/of/key"
    pub fn from_str(name: &str) -> Option<Self> {

        let mut parts = name.trim_matches('/').splitn(2, '/');
        let bucket = parts.next().unwrap_or("").to_string();
        let key = parts.next().unwrap_or("").trim_matches('/').to_string();
        if bucket == "" {
            None
        } else {
            Some(Self { bucket, key })
        }
    }
}



/// Creates a connection to an S3 server
#[no_mangle]
pub unsafe extern "C" fn s3_connect() -> *mut Client {
    let result =
        rt().unwrap().block_on(async {
            let config = aws_config::load_from_env().await;
            Client::new(&config)
        });
    Box::into_raw(Box::new(result))
}

/// Destroys a connection to an S3 server
#[no_mangle]
pub unsafe extern "C" fn s3_disconnect(client: *mut Client) {
    if !client.is_null() {
        drop(Box::from_raw(client));
    }
}



/// Checks if a file exists on the server. Returns 0 if the object
/// exists, -1 if it does not exist.
#[no_mangle]
pub unsafe extern "C" fn s3_file_exists(client: *const Client,
                                        file_name: *const c_char,
                                        str_len: usize) -> i32 {
    if client.is_null() {
        panic!("In s3_file_exists, client is a NULL pointer.")
    };

    if file_name.is_null() {
        panic!("In s3_file_exists, file_name is a NULL pointer.")
    };

    let cstr = CStr::from_ptr(file_name);
    if cstr.to_bytes().len() > str_len {
        panic!("In s3_file_exists, file_name length too small.")
    };

    let s: &str = cstr.to_str().unwrap();

    let b = BucketAndKey::from_str(s);
    match b {
        None => -1,
        Some(b) => {
            let bucket = b.bucket;
            let key = b.key;

            let c = &(*client);
            // Attempt to fetch object metadata
            rt().unwrap().block_on(async {
                match c.head_object().bucket(bucket).key(key).send().await {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            })
        }
    }
}


#[no_mangle]
pub unsafe extern "C" fn s3_size(client: *const Client,
                                 file_name: *const c_char,
                                 str_len: usize) -> i64
{
    if client.is_null() {
        panic!("In s3_size, client is a NULL pointer.")
    };

    if file_name.is_null() {
        panic!("In s3_size, file_name is a NULL pointer.")
    };

    let cstr = CStr::from_ptr(file_name);
    if cstr.to_bytes().len() > str_len {
        panic!("In s3_size, file_name length too small.")
    };

    let s: &str = cstr.to_str().unwrap();
    let b = BucketAndKey::from_str(s);
    match b {
        None => -1,
        Some(b) => {
            let bucket = b.bucket;
            let key = b.key;

            let c = &(*client);
            // Attempt to fetch object metadata
            rt().unwrap().block_on(async {
                // Perform a HEAD request to get object metadata
                let resp = match c.head_object().bucket(bucket).key(key).send().await {
                    Ok(r) => r,
                    _ => return -1
                };

                resp.content_length().unwrap()
            })
        }
    }
}



/// Retreives the content of an object into a buffer. Returns 0 upon
/// success.
#[no_mangle]
pub unsafe extern "C" fn s3_get(client: *const Client,
                                file_name: *const c_char,
                                str_len: usize,
                                buffer: *mut u8,
                                buffer_size: usize) -> i32
{
    if client.is_null() {
        panic!("In s3_get, client is a NULL pointer.")
    };

    if file_name.is_null() {
        panic!("In s3_get, file_name is a NULL pointer.")
    };

    if buffer.is_null() {
        panic!("In s3_get, buffer is a NULL pointer.")
    };

    let cstr = CStr::from_ptr(file_name);
    if cstr.to_bytes().len() > str_len {
        panic!("In s3_get, file_name length too small.")
    };

    let s: &str = cstr.to_str().unwrap();
    let b = BucketAndKey::from_str(s);
    match b {
        None => -1,
        Some(b) => {
            let bucket = b.bucket;
            let key = b.key;

            let c = &(*client);
            // Attempt to fetch object metadata
            rt().unwrap().block_on(async {
                let mut object =
                    match c.get_object().bucket(bucket).key(key).send().await {
                        Ok(o) => o,
                        Err(_) => return -1,
                    };
                unsafe {
                    let dest_slice: &mut [u8] = slice::from_raw_parts_mut(buffer, buffer_size);

                    let mut total_bytes_read = 0_usize;
                    while let Some(bytes) = object.body.try_next().await.unwrap() {
                        let bytes_len: usize = bytes.len().try_into().unwrap();
                        if total_bytes_read + bytes_len > buffer_size {
                            return -1
                        };
                        for i in 0..bytes_len {
                            dest_slice[total_bytes_read+i] = bytes[i];
                        }
                        total_bytes_read += bytes_len;
                    };
                }
                0
            })
        }
    }
}



/// Puts the content of a buffer into an object. Returns 0 upon
/// success.
#[no_mangle]
pub unsafe extern "C" fn s3_put(client: *const Client,
                                file_name: *const c_char,
                                str_len: usize,
                                buffer: *const u8,
                                buffer_size: usize) -> i32
{
    if client.is_null() {
        panic!("In s3_put, client is a NULL pointer.")
    };

    if file_name.is_null() {
        panic!("In s3_put, file_name is a NULL pointer.")
    };

    if buffer.is_null() {
        panic!("In s3_put, buffer is a NULL pointer.")
    };

    let cstr = CStr::from_ptr(file_name);
    if cstr.to_bytes().len() > str_len {
        panic!("In s3_put, file_name length too small.")
    };

    let s: &str = CStr::from_ptr(file_name).to_str().unwrap();
    let b = BucketAndKey::from_str(s);
    match b {
        None => -1,
        Some(b) => {
            let bucket = b.bucket;
            let key = b.key;

            let c = &(*client);
            let s = unsafe {slice::from_raw_parts(buffer, buffer_size) };
            let body = aws_sdk_s3::primitives::ByteStream::from_static(s);

            // Attempt to fetch object metadata
            rt().unwrap().block_on(async {
                match c.put_object()
                    .bucket(bucket)
                    .key(key)
                    .body(body)
                    .send()
                    .await {
                        Ok(_) => 0,
                        _ => -1
                    }
            })
        }
    }
}

// ----------------- Tests -----------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn str_to_c_char(s: &str) -> CString {
        CString::new(s).expect("CString::new failed")
    }

    #[test]
    fn connect() {
        let client = unsafe {Box::from_raw( s3_connect() )} ;
        let response = rt().unwrap().block_on(async {
            client.list_buckets().send().await.unwrap() });
        match response.buckets {
            None => panic!("No response"),
            Some(buckets) =>
                for i in 0..buckets.len() {
                    println!( "{:?}", buckets[i].name)
                }
        };
    }

    #[test]
    fn bucket_and_key() {
       let b = BucketAndKey::from_str("/hello/world/pouet").unwrap();
       assert_eq!(b.bucket, "hello");
       assert_eq!(b.key, "world/pouet");

       let b = BucketAndKey::from_str("/hello/world/pouet/").unwrap();
       assert_eq!(b.bucket, "hello");
       assert_eq!(b.key, "world/pouet");

       let b = BucketAndKey::from_str("hello/world/pouet").unwrap();
       assert_eq!(b.bucket, "hello");
       assert_eq!(b.key, "world/pouet");

       let b = BucketAndKey::from_str("///hello///world/pouet///").unwrap();
       assert_eq!(b.bucket, "hello");
       assert_eq!(b.key, "world/pouet");

       let b = BucketAndKey::from_str("hello/").unwrap();
       assert_eq!(b.bucket, "hello");
       assert_eq!(b.key, "");

       let b = BucketAndKey::from_str("");
       match b {
           None => (),
           _ => assert!(false),
       }
    }

    #[test]
    fn file_exists() {
        unsafe {
            let client = s3_connect();

            let s = "poc-trexio-b1/test";
            let x = str_to_c_char(s);
            let file_name = x.as_ptr();
            let response = s3_file_exists(client, file_name,s.len());
            assert_eq!(response, 0);

            let s = "poc-trexio-b1/test_c";
            let x = str_to_c_char(s);
            let file_name = x.as_ptr();
            let response = s3_file_exists(client, file_name,s.len());
            assert_eq!(response, 0);

            let s = "poc-trexio-b1/test.c";
            let x = str_to_c_char(s);
            let file_name = x.as_ptr();
            let response = s3_file_exists(client, file_name, s.len());
            assert_eq!(response, -1);

            let s = "poc-trexio-b1/test/hello.rs";
            let x = str_to_c_char(s);
            let file_name = x.as_ptr();
            let response = s3_file_exists(client, file_name, s.len());
            assert_eq!(response, -1);
        }
    }

    fn put_object() {
        unsafe {
            let client = s3_connect();

            let s = "poc-trexio-b1/test";
            let x = str_to_c_char(s);
            let file_name = x.as_ptr();
            let string : Vec::<u8> = "Hello world!".bytes().collect();
            let ptr: *const u8 = string.as_slice().as_ptr() as *const u8;
            let rc = s3_put(client, file_name, s.len(), ptr, string.len());
            assert_eq!(rc, 0);
        }
    }

    fn get_object() {
        unsafe {
            let client = s3_connect();

            let s = "poc-trexio-b1/test";
            let x = str_to_c_char(s);
            let file_name = x.as_ptr();
            let size = s3_size(client, file_name, s.len());
            assert!(size > 0);

            let size = size.try_into().unwrap();
            let mut buffer = vec![0_u8 ; size];
            let ptr: *mut u8 = buffer.as_mut_slice().as_mut_ptr();
            let rc = s3_get(client, file_name, s.len(), ptr, size);
            assert_eq!(rc, 0);
            assert_eq!(String::from_utf8(buffer).unwrap(), String::from("Hello world!"));
        }
    }

    #[test]
    fn put_and_get() {
        put_object();
        get_object();
    }

}
