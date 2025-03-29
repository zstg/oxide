// Quoted from oxide
// > How to run:
// >
// >   $ ./oxide examples/mat.c > tmp-mat.s
// >   $ gcc -static -o tmp-mat tmp-mat.s
// >   $ ./tmp-mat

int main() {
    int A[2][2] = {{1, 2}, {3, 4}};
    int B[2][2] = {{5, 6}, {7, 8}};
    int C[2][2];
    
    for (int i = 0; i < 2; i++)
        for (int j = 0; j < 2; j++)
            C[i][j] = A[i][0] * B[0][j] + A[i][1] * B[1][j];
}
