mod fs;
mod trans;

use trans::Trans;

fn main() {
    let mut trans = Trans::new();
    trans.add_arg("hello".as_bytes().to_vec());
    println!("trans {:?} arg mode={}", trans, trans.arg_mode());
    let args = trans.take_args(2);
    println!("args {:?}", args);

    trans.add_arg("world".as_bytes().to_vec());
    trans.set_resp("HELLO".as_bytes().to_vec());
    for _ in 0..3 {
        let bs = &trans.read_resp(3);
        let d = String::from_utf8_lossy(&bs);
        println!(
            "trans {:?} arg mode={}, data={}",
            trans,
            trans.arg_mode(),
            d
        );
    }
    let args = trans.take_args(2);
    println!("args {:?}", args);

    //let mut fs = fs::Fs::new();
    if true {
        let mut fs = fs::Fs::new_test();
        fs.test_walk("/dir1/f1");
        fs.test_walk("dir1/f1");
        fs.test_walk("/dir1/../dir2");
        fs.test_walk("//dir2/.././/dir1/f1");

        fs.test_walk("/bogus");
        fs.test_walk("/dir1/f1/bogus");
    }

    //println!("fs {:?}", fs);
}
