class LoopContinue {
    static function main() {
        var b = 69;
        while (b < 1024) {
            if (b - 2 == 0) {
                continue;
            }
            b += 4;
        }
    }
}