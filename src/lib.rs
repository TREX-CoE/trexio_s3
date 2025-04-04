use tokio::runtime;
use aws_sdk_s3::Client;


// Found on Reddit: https://www.reddit.com/r/rust/comments/18xrp8m/comment/kgj5uhc/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button
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
struct BucketAndKey {
    bucket: String,
    key: String,
}

impl BucketAndKey {

    /// Creates a BucketAndKey from a string. For example: /bucket/path/of/key -> "bucket" and
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
pub unsafe extern "C" fn s3_connect() -> *mut Client {
    let result = 
      rt().unwrap().block_on(async {
          let config = aws_config::load_from_env().await;
          Client::new(&config)
      });
    Box::into_raw(Box::new(result))
}

/// Destroys a connection to an S3 server
pub unsafe extern "C" fn s3_disconnect(client: *mut Client) {
    drop(Box::from_raw(client));
}



pub unsafe extern "C" fn s3_file_exists(s3_handle: *mut Client, file_name: &str) -> bool {
  true
}


#[cfg(test)]
mod tests {
    use super::*;

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
}
