pub struct TablePrinter {
    cols: Vec<String>,
    rows: Vec<Vec<String>>,
}

impl TablePrinter {
    pub fn new(cols: Vec<String>) -> TablePrinter {
        TablePrinter {
            cols,
            rows: Vec::new(),
        }
    }

    pub fn add_row(&mut self, values: Vec<String>) -> Result<(), &'static str> {
        if values.len() > self.cols.len() {
            return Err("Too many values");
        }

        self.rows.push(values);

        return Ok(());
    }

    pub fn print(&self) {
        let mut col_sizes = Vec::new();
        for i in 0..self.cols.len() {
            col_sizes.push(self.get_column_size(i))
        }

        let mut buf = Vec::with_capacity(self.cols.len());
        // print columns
        for (i, c) in self.cols.iter().enumerate() {
            buf.push(format!("{:<1$}", c, col_sizes[i]))
        }
        println!("{}", buf.join("    "));
        buf.clear();

        // print rows
        for r in self.rows.iter() {
            for (i, v) in r.iter().enumerate() {
                buf.push(format!("{:<1$}", v, col_sizes[i]));
            }

            println!("{}", buf.join("    "));
            buf.clear();
        }
    }

    fn get_column_size(&self, col: usize) -> usize {
        let mut max_found = self.cols[col].len();

        for r in self.rows.iter() {
            let l = r[col].len();
            if l > max_found {
                max_found = l;
            }
        }

        return max_found;
    }
}
