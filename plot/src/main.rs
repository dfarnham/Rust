use clap::Parser;
use general::read_trimmed_data_lines;
use rgb::RGB8;
use textplots::{Chart, ColorPlot, Shape, TickDisplay, TickDisplayBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Parser, Debug)]
    #[clap(author, version, about, long_about=None)]
    struct Args {
        /// file|stdin, filename of "-" implies stdin
        file: Option<std::path::PathBuf>,
    }

    // read the values from the file
    let args = Args::parse();
    let values = read_trimmed_data_lines::<f32>(args.file.as_ref())?;

    // enumerate the values into a list of tuples (f32, f32)
    let points = values
        .iter()
        .enumerate()
        .map(|(i, p)| (i as f32, *p))
        .collect::<Vec<_>>();

    // draw the plot
    Chart::new(220, 100, 0.0, points.len() as f32)
        .linecolorplot(&Shape::Lines(&points), RGB8 { r: 0, g: 255, b: 0 })
        .y_tick_display(TickDisplay::Sparse)
        .nice();

    Ok(())
}
