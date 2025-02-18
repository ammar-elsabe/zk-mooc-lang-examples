pragma circom 2.0.0;

template NonEqual(){
    signal input in0;
    signal input in1;
    signal inv;
    inv <-- 1/ (in0 - in1);
    inv*(in0 - in1) === 1;
}

template Distinct(n) {
    signal input in[n];
    component nonEqual[n][n];
    for(var i = 0; i < n; i++){
        for(var j = 0; j < i; j++){
            nonEqual[i][j] = NonEqual();
            nonEqual[i][j].in0 <== in[i];
            nonEqual[i][j].in1 <== in[j];
        }
    }
}

// Enforce that 0 <= in < 16
template Bits4(){
    signal input in;
    signal bits[4];
    var bitsum = 0;
    for (var i = 0; i < 4; i++) {
        bits[i] <-- (in >> i) & 1;
        bits[i] * (bits[i] - 1) === 0;
        bitsum = bitsum + 2 ** i * bits[i];
    }
    bitsum === in;
}

// Enforce that 1 <= in <= 9
template OneToNine() {
    signal input in;
    component lowerBound = Bits4();
    component upperBound = Bits4();
    lowerBound.in <== in - 1;
    upperBound.in <== in + 6;
}

template Sudoku(n) {
    // solution is a 2D array: indices are (row_i, col_i)
    signal input solution[n][n];
    // puzzle is the same, but a zero indicates a blank
    signal input puzzle[n][n];

    component distinct_rows[n];
    component distinct_cols[n];
    ////// overlapping subgrids, the constraint is non overlapping
    // component distinct_grids[n - 2][n - 2];
    component distinct_grids[n - 3][n - 3];
    component inRange[n][n];

    for (var row_i = 0; row_i < n; row_i++) {
        for (var col_i = 0; col_i < n; col_i++) {
            // this is to ensure that the solution contains the same pre filled values
            // either the puzzle          
            // was empty at the cell   or the solution is the same as the puzzle at the cell
            puzzle[row_i][col_i]    * (puzzle[row_i][col_i] - solution[row_i][col_i]) === 0;
        }
    }

    // all elements in range and
    // all rows are distinct
    for (var row_i = 0; row_i < n; row_i++) {
        distinct_rows[row_i] = Distinct(n);
        for (var col_i = 0; col_i < n; col_i++) {
            inRange[row_i][col_i] = OneToNine();
            inRange[row_i][col_i].in <== solution[row_i][col_i];
            distinct_rows[row_i].in[col_i] <== solution[row_i][col_i];
        }
    }

    // all columns are distinct
    for (var col_i = 0; col_i < n; col_i++) {
        distinct_cols[col_i] = Distinct(n);
        for (var row_i = 0; row_i < n; row_i++) {
            distinct_cols[col_i].in[row_i] <== solution[row_i][col_i];
        }
    }

    ////// overlapping subgrids, the constraint is non overlapping
    // all subgrids are distinct
    // for (var row_i = 0; row_i < n - 2; row_i++) {
    //     for (var col_i = 0; col_i < n - 2; col_i++) {
    //         distinct_grids[row_i][col_i] = Distinct(9);
    //         for (var row_j = 0; row_j < 3; row_j++) {
    //             for (var col_j = 0; col_j < 3; col_j++) {
    //               distinct_grids[row_i][col_i].in[row_j * 3 + col_j] <== solution[row_i + row_j][col_i + col_j];
    //             }
    //         }
    //     }
    // }

    // all subgrids are distinct
    for (var row_i = 0; row_i < n; row_i++) {
      for (var col_i = 0; col_i < n; col_i++) {
        if (row_i % 3 == 0 && col_i % 3 == 0) {
          distinct_grids[row_i / 3][col_i / 3] = Distinct(9);
          for (var row_j = 0; row_j < 3; row_j++) {
            for (var col_j = 0; col_j < 3; col_j++) {
              distinct_grids[row_i / 3][col_i / 3].in[row_j * 3 + col_j] <== solution[row_i + row_j][col_i + col_j];
            }
          }
        }
      }
    }
}

component main {public[puzzle]} = Sudoku(9);
