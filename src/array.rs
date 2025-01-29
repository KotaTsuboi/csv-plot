use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;

// 二次元配列（ジェネリクス版）
#[derive(Debug)]
pub struct TwoDimentionalArray<T> {
    headers: Vec<String>, // 各列の名前（先頭行の各セル）
    index: Vec<f32>,      // 各行の名前（各行の先頭セル）
    row_size: usize,      // 行数
    col_size: usize,      // 列数
    dat: Vec<T>,          // 配列のデータ
}
impl<T: Clone> TwoDimentionalArray<T> {
    // at(i,j) は i 行 j 列の要素を取り出す
    pub fn at(&self, i: usize, j: usize) -> Result<T> {
        if i >= self.row_size || j >= self.col_size {
            return Err(Error::new(ErrorKind::Other, "abnormal index!"));
        }
        Ok(self.dat[i * self.row_size + j].clone())
    }

    // col(j) は j 列の要素のベクタを取り出す。j が異常値のときは空のベクタを返す
    pub fn col(&self, j: usize) -> Vec<T> {
        let mut r: Vec<T> = vec![];
        if j < self.col_size {
            for (pos, v) in self.dat.iter().enumerate() {
                if pos % self.col_size == j {
                    r.push((*v).clone());
                }
            }
        }
        r
    }

    // row(i) は i 行の要素のベクタを取り出す。i が異常値のときは空のベクタを返す
    pub fn row(&self, i: usize) -> Vec<T> {
        let mut r: Vec<T> = vec![];
        if i < self.row_size {
            for (pos, v) in self.dat.iter().enumerate() {
                if pos / self.col_size == i {
                    r.push((*v).clone());
                }
            }
        }
        r
    }

    // 各行の名前のベクタを返す
    pub fn index(&self) -> Vec<f32> {
        self.index.clone()
    }

    // 各列の名前のベクタを返す
    pub fn headers(&self) -> Vec<String> {
        self.headers.clone()
    }

    // データのベクタを返す
    pub fn dat(&self) -> Vec<T> {
        self.dat.clone()
    }

    // データのイテレータを返す
    pub fn iter(&self) -> core::slice::Iter<T> {
        self.dat.iter()
    }

    pub fn row_size(&self) -> usize {
        self.row_size
    }

    pub fn col_size(&self) -> usize {
        self.col_size
    }

    pub fn size(&self) -> usize {
        self.row_size * self.col_size
    }
}

// CSV ファイルを読みだす関数
pub fn from_csv_file<S: std::str::FromStr>(csv_file: &str) -> Result<TwoDimentionalArray<S>> {
    let mut dat: Vec<S> = vec![];

    let file = std::fs::File::open(csv_file)?;
    let mut rdr = csv::Reader::from_reader(file);

    // ヘッダの取得。CSVファイルの先頭行は常にヘッダとみなされることに注意。
    let line = rdr.headers()?;
    let headers: Vec<String> = line.iter().skip(1).map(|x| String::from(x)).collect();
    let col_size = headers.len();

    let mut index: Vec<f32> = vec![];
    let mut row_size: usize = 0;

    for line in rdr.records() {
        let line = line?;

        // ヘッダの列数と合わないときはエラー
        if line.len() - 1 != col_size {
            return Err(Error::new(ErrorKind::Other, "column size is missmatched!"));
        }

        let mut iter = line.iter();

        // 先頭セルは行の名前
        index.push(iter.next().unwrap().parse::<f32>().unwrap());

        for item in iter {
            if let Ok(value) = item.parse() {
                dat.push(value);
            } else {
                // [MEMO] parse のエラー std::str::FromStr::Err から
                // std::io::Error に変換する From トレイトが実装されていないことに注意
                return Err(Error::new(ErrorKind::Other, "failed in parsing!"));
            }
        }
        row_size += 1;
    }

    Ok(TwoDimentionalArray {
        headers,
        index,
        row_size,
        col_size,
        dat,
    })
}

// 二次元配列の表示
impl<T: std::fmt::Debug> std::fmt::Display for TwoDimentionalArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[ ")?;
        for (i, x) in self.dat.iter().enumerate() {
            write!(f, "{:?} ", x)?;
            if i == self.col_size * self.row_size - 1 {
                write!(f, "]")?;
            } else if (i + 1) % self.col_size == 0 {
                write!(f, "\n  ")?;
            }
        }
        Ok(())
    }
}
