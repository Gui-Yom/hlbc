class BranchNested {
    static function main() {
        var a = 0;
        if (a > 1) {
            a = 1;
            if (a == 10) {
                a = 42;
                return;
            } else {
                a = 41;
            }
            a = 4;
            var cond = a != 3;
            if (cond) {
                a = 16;
            }
        } else if (a <= 2) {
            a = 2;
            return;
        }
        a = 3;
    }
}
