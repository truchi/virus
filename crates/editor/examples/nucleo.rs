use nucleo::{
    pattern::{CaseMatching, Normalization},
    Config, Nucleo,
};
use std::sync::Arc;

fn main() {
    let notify = || {
        println!(">Notify<");
    };
    let notify = Arc::new(notify);
    let mut nucleo = Nucleo::<usize>::new(Config::DEFAULT, notify, None, 1);

    let injector = nucleo.injector();

    let strs = ["World", "LOL", "axbycz", "HiEmLaLiOu", "Hello"];

    println!("Pushing");
    for (i, str) in strs.into_iter().enumerate() {
        injector.push(i, |_value, columns| {
            for column in columns {
                *column = str.into();
            }
        });
    }

    let lesgo = |nucleo: &mut Nucleo<usize>, needle: &str| {
        println!("Reparse");
        nucleo
            .pattern
            .reparse(0, needle, CaseMatching::Smart, Normalization::Smart, false);

        loop {
            println!("Tick");
            let status = nucleo.tick(10);

            if status.changed {
                for item in nucleo.snapshot().matched_items(..) {
                    let matched = item.matcher_columns[0].to_string();
                    dbg!(matched);
                }
            }

            if !status.running {
                break;
            }
        }
    };

    lesgo(&mut nucleo, "hello");
    lesgo(&mut nucleo, "abc");
    lesgo(&mut nucleo, "xyz");
}
