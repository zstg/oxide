int fib(int n) {
	int fibdp[64] = {0};
    if (n == 0 || n == 1) {
        return n;
    }
    if (fibdp[n] != 0) {
        return fibdp[n];
    }
    return fibdp[n] = fib(n - 2) + fib(n - 1);
}

int main() {
    int ans = fib(46);
    return 0;
}
