class LoopForEach {
    static function main() {
        var sum = 0;
        for (i in items()) sum += i;
    }

    static function items() {
        return [1, 2, 3];
    }
}