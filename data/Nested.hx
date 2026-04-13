class Nested {
    static function main() {
        var b = 512.0;
        while (b > 5) {
            b /= 2;
            if (b < 10) {
                while (b < 100) {
                    b *= 2;
                    if (b > 50) {
                        break;
                    }
                }
            }
        }
    }
}
