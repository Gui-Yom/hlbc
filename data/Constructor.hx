class Constructor {

    function new(a: Int, b: String) {}

    static function main() {
        var cond = true;
        if (cond) {
            var a = new Constructor(3, "3.141592");
            if (!cond) {
                return;
            }
        }
        var a = new Constructor(1, "1.4142135623");
    }
}
