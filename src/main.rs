use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// 出力するグラフ画像のサイズ
    #[arg(long, default_value_t = 512)]
    image_width: u32,
    #[arg(long, default_value_t = 384)]
    image_height: u32,

    /// 出力するグラフ画像のキャプション・フォント・サイズ
    #[arg(long, default_value = "")]
    caption: String,
    #[arg(long, default_value = "sans-serif")]
    font_face: String,
    #[arg(long, default_value_t = 10)]
    font_size: i32,

    /// 上下左右全ての余白
    #[arg(long, default_value_t = 10)]
    margin: i32,
    /// x軸ラベル部分の余白
    #[arg(long, default_value_t = 40)]
    x_label_area_size: i32,
    /// y軸ラベル部分の余白
    #[arg(long, default_value_t = 40)]
    y_label_area_size: i32,

    #[arg(long, default_value = "")]
    x_desc: String,
    #[arg(long, default_value = "")]
    y_desc: String,
    #[arg(long, default_value = "sans_serif")]
    axis_font_face: String,
    #[arg(long, default_value_t = 10)]
    axis_font_size: i32,

    /// 点のサイズ
    #[arg(long, default_value_t = 1)]
    circle_size: i32,

    #[arg(long, default_value = "x")]
    x_operation: String,
    #[arg(long, default_value = "y")]
    y_operation: String,

    #[arg(long, value_delimiter = ',')]
    y_mark: Vec<f32>,

    #[arg(long, short)]
    input: String,
    #[arg(long, short)]
    output: String,
}

use array::TwoDimentionalArray;
use plotters::prelude::*;

mod array;

use evalexpr::error::EvalexprError::ExpectedFloat;
use evalexpr::Value::Int;
use evalexpr::*;

fn operate(expr: &str, variable: &str, data: Vec<f32>) -> Vec<f32> {
    let mut after = Vec::new();

    for x in data {
        let expr = format!("{variable} = {x}; {expr}");
        let result = match eval_float(&expr) {
            Ok(result) => result,
            Err(e) => match e {
                ExpectedFloat { actual: Int(i) } => i as f64,
                _ => panic!(""),
            },
        };
        after.push(result as f32);
    }

    after
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::parse();

    // (1) プロット用データの準備

    // CSV ファイルから読み出す
    let data: TwoDimentionalArray<f32> = array::from_csv_file(&config.input)?;

    // x軸
    let x_strategy = ValueStrategy::new();
    // x軸：日付の系列
    let xs = x_strategy.series(&data.index())?;
    let xs = operate(&config.x_operation, "x", xs);
    // x軸の値の範囲
    let x_range = x_strategy.range(&xs)?;

    // y軸
    let y_strategy = ValueStrategy::new();
    // y軸：値の系列
    let ys = y_strategy.series(&data.dat())?;
    let ys = operate(&config.y_operation, "y", ys);
    // y軸の値の範囲
    let y_range = y_strategy.range(&ys)?;

    // (2) 描画先の情報を設定

    // 描画先を指定。画像出力する場合はBitMapBackend
    let root = SVGBackend::new(&config.output, (&config.image_width, &config.image_height))
        .into_drawing_area();

    // 背景を白にする
    root.fill(&WHITE)?;

    // (3) グラフ全般の設定
    let font = (&config.font_face, &config.font_size);

    let mut chart = ChartBuilder::on(&root)
        .caption(&config.caption, font.into_font())
        .margin(&config.margin)
        .x_label_area_size(&config.x_label_area_size)
        .y_label_area_size(&config.y_label_area_size)
        .build_cartesian_2d(
            // x軸とy軸の値の範囲を指定
            x_range, y_range,
        )?;

    // (4) グラフの描画

    // x軸y軸、グリッド線などを描画
    //chart.configure_mesh().draw()?;

    let axis_desc_style = (axis_font_face, axis_font_size);

    chart
        .configure_mesh()
        .x_label_formatter(&|x: &f32| x.to_string())
        .x_desc(&config.x_desc)
        .y_desc(&config.y_desc)
        .axis_desc_style(axis_desc_style)
        .draw()?;

    // 折れ線グラフの描画
    let line_series = LineSeries::new(xs.iter().zip(ys.iter()).map(|(x, y)| (*x, *y)), &RED);
    chart.draw_series(line_series)?;

    // 点グラフの描画
    let point_series = xs.iter().zip(ys.iter()).map(|(x, y)| {
        Circle::new(
            (*x, *y),
            &config.circle_size,
            RED, // 色を指定
                 // ↓円を塗りつぶしたければこちら
                 // ShapeStyle::from(&RED).filled(),
        )
    });
    chart.draw_series(point_series)?;

    let x_min = chart.x_range().start;
    let x_max = chart.x_range().end;

    for y in &config.y_mark {
        let points = vec![(x_min, y), (x_max, y)];

        let style = ShapeStyle {
            color: RED.to_rgba(),
            filled: true,
            stroke_width: 1,
        };

        let path_element = PathElement::new(points, style);

        let mark_series = std::iter::once(path_element);

        chart.draw_series(mark_series)?;
    }

    Ok(())
}

// 軸の処理に関する抽象化
use core::ops::Range;
trait AxisStrategy<S, T> {
    // データ系列の作成
    fn series(&self, series: &Vec<S>) -> Result<Vec<T>, Box<dyn std::error::Error>>;

    // データ範囲の作成
    fn range(&self, series: &Vec<T>) -> Result<Range<T>, Box<dyn std::error::Error>>;
}

struct ValueStrategy {}
impl ValueStrategy {
    fn new() -> Self {
        ValueStrategy {}
    }
}

impl AxisStrategy<f32, f32> for ValueStrategy {
    fn series(&self, series: &Vec<f32>) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        Ok(series.to_vec())
    }

    fn range(&self, ys: &Vec<f32>) -> Result<Range<f32>, Box<dyn std::error::Error>> {
        let (y_min, y_max) = ys
            .iter()
            .fold((f32::NAN, f32::NAN), |(m, n), v| (v.min(m), v.max(n)));
        // [MEMO]
        // y軸の最大最小値を算出
        // f32型はNaNが定義されていてys.iter().max()等が使えないので工夫が必要
        // 参考サイト
        // https://qiita.com/lo48576/items/343ca40a03c3b86b67cb

        Ok(y_min..y_max)
    }
}
