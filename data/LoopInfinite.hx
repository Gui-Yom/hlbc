class LoopInfinite {
    static function main() {
        var b = 69;
        while (true) {
            b *= 2;
            if (b > 1000) {
                break;
            }
        }
    }
}