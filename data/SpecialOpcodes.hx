class SpecialOpcodes {
    static function main() {
        var a = 0;
        untyped $prefetch(a, 0);
        untyped $asm(3, 1, a);
        trace(a);
    }
}