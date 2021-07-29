template ManyConstraints() {
    signal private input a;
    signal output c;

    signal b;

    c <== a;
    signal d[10000];
    d[0] <== b;
    for (var i = 0; i < 10000-1; i++) {
        b <== d[i] * d[i];
        d[i+1] <== c * d[i];
    }
}

component main = ManyConstraints();
