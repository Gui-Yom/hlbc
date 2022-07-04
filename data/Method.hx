class Method {

    function new() {}

    function a() {
        return 42;
    }

    static function main() {
        var instance = new Method();
        var a = instance.a();
    }
}