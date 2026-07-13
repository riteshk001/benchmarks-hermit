 //matrix multiplication

pub fn do_work(work_units: u64) {
    let n = work_units as usize;

    let mut a = vec![0.0_f64; n * n];
    let mut b = vec![0.0_f64; n * n];
    let mut c = vec![0.0_f64; n * n];

    for i in 0..n * n {
        a[i] = (i as f64 % 100.0) * 0.5;
        b[i] = (i as f64 % 50.0) * 0.25;
    }

    for i in 0..n {
        for k in 0..n {
            let aik = a[i * n + k];

            for j in 0..n {
                c[i * n + j] += aik * b[k * n + j];
            }
        }
    }

    std::hint::black_box(c);
}