// 出力するグラフ画像のサイズ
const IMAGE_WIDTH: u32 = 1024;
const IMAGE_HEIGHT: u32 = 768;

// 出力するグラフ画像のキャプション・フォント・サイズ
const CAPTION: &str = "P-θ";
const FONT_FACE: &str = "sans-serif";
const FONT_SIZE: i32 = 20;

// 上下左右全ての余白
const MARGIN: i32 = 10;
// x軸ラベル部分の余白
const X_LABEL_AREA_SIZE: i32 = 50;
// y軸ラベル部分の余白
const Y_LABEL_AREA_SIZE: i32 = 50;

const X_DESC: &str = "変形角";
const Y_DESC: &str = "荷重[kN]";
const AXIS_FONT_FACE: &str = "sans-serif";
const AXIS_FONT_SIZE: i32 = 20;

// 点のサイズ
const CIRCLE_SIZE: i32 = 2;

const X_OPERATION: &str = "x/1339.0";
const Y_OPERATION: &str = "y";

const Y_MARK: [f32; 2] = [200.0, 300.0];

use array::TwoDimentionalArray;
use plotters::prelude::*;

mod array;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("[Usage] {} data.csv out.svg", args[0]);
        return;
    }

    let csv_file = &args[1];
    let out_file = &args[2];

    if let Err(e) = process(csv_file, out_file) {
        eprintln!("[Error] {}", e);
    }
}

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

fn process(csv_file: &str, out_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // (1) プロット用データの準備

    // CSV ファイルから読み出す
    let data: TwoDimentionalArray<f32> = array::from_csv_file(csv_file)?;

    // x軸
    let x_strategy = ValueStrategy::new();
    // x軸：日付の系列
    let xs = x_strategy.series(&data.index())?;
    let xs = operate(X_OPERATION, "x", xs);
    // x軸の値の範囲
    let x_range = x_strategy.range(&xs)?;

    // y軸
    let y_strategy = ValueStrategy::new();
    // y軸：値の系列
    let ys = y_strategy.series(&data.dat())?;
    let ys = operate(Y_OPERATION, "y", ys);
    // y軸の値の範囲
    let y_range = y_strategy.range(&ys)?;

    // (2) 描画先の情報を設定

    // 描画先を指定。画像出力する場合はBitMapBackend
    let root = SVGBackend::new(out_file, (IMAGE_WIDTH, IMAGE_HEIGHT)).into_drawing_area();

    // 背景を白にする
    root.fill(&WHITE)?;

    // (3) グラフ全般の設定
    let font = (FONT_FACE, FONT_SIZE);

    let mut chart = ChartBuilder::on(&root)
        .caption(CAPTION, font.into_font())
        .margin(MARGIN)
        .x_label_area_size(X_LABEL_AREA_SIZE)
        .y_label_area_size(Y_LABEL_AREA_SIZE)
        .build_cartesian_2d(
            // x軸とy軸の値の範囲を指定
            x_range, y_range,
        )?;

    // (4) グラフの描画

    // x軸y軸、グリッド線などを描画
    //chart.configure_mesh().draw()?;

    let axis_desc_style = (AXIS_FONT_FACE, AXIS_FONT_SIZE);

    chart
        .configure_mesh()
        .x_label_formatter(&|x: &f32| x.to_string())
        .x_desc(X_DESC)
        .y_desc(Y_DESC)
        .axis_desc_style(axis_desc_style)
        .draw()?;

    // 折れ線グラフの描画
    let line_series = LineSeries::new(xs.iter().zip(ys.iter()).map(|(x, y)| (*x, *y)), &RED);
    chart.draw_series(line_series)?;

    // 点グラフの描画
    let point_series = xs.iter().zip(ys.iter()).map(|(x, y)| {
        Circle::new(
            (*x, *y),
            CIRCLE_SIZE,
            RED, // 色を指定
                 // ↓円を塗りつぶしたければこちら
                 // ShapeStyle::from(&RED).filled(),
        )
    });
    chart.draw_series(point_series)?;

    let x_min = chart.x_range().start;
    let x_max = chart.x_range().end;

    for y in Y_MARK {
        let points = vec![(x_min, y), (x_max, y)];

        let style = ShapeStyle {
            color: RED.to_rgba(),
            filled: true,
            stroke_width: 2,
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
