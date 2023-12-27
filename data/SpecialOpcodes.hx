class SpecialOpcodes {
    static function main() {
        var a = 0;
        // https://haxe.org/manual/target-syntax.html#other-platforms
        untyped $prefetch(a, 0);
        untyped $asm(3, 1, a);
        trace(a);
    }
}