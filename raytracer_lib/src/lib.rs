#![feature(exclusive_range_pattern)]

use proc_macro::TokenStream;
use rand::distributions::{Distribution, Uniform};

#[proc_macro]
pub fn generate_multisample_positions(size_tokens: proc_macro::TokenStream) -> TokenStream {
    let str = size_tokens.to_string();
    let size = str.parse::<usize>().unwrap();

    let mut output_arr = Vec::<(f32, f32)>::with_capacity(size);

    let between = Uniform::from(0.0..1.0);
    let mut rng_x = rand::thread_rng();
    let mut rng_y = rand::thread_rng();

    for _ in 0..size {
        output_arr.push(
            (
                between.sample(&mut rng_x),
                between.sample(&mut rng_y)
            )
        );
    }

    let mut output_stream: String = format!("const fn generated_samples() -> [(f32,f32); {size}]");
    output_stream.push_str("{");
    output_stream.push_str("return [");
    for point in output_arr {
        output_stream.push_str(format!("({}, {}),", point.0, point.1).as_str());
    }
    output_stream.push_str("]}");
    return output_stream.parse().unwrap();
}
