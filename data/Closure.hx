class Closure {
    static function main() {
        var fun = () -> "hello";
        trace(fun());
        trace(fun);
        // This will get inlined
        (() -> { trace(" there"); })();
    }
}