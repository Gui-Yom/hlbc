class Branch {
    static function main() {
        var cond = true;
        if (cond) {
            var a = 5;
            if (!cond) {
                a = 6;
                return;
            }
        }
    }
}
