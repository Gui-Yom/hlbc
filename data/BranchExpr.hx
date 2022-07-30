class BranchExpr {
    static function main() {
        var a = true;
        var b = if (a) {
            3;
        } else {
            2;
        };
    }
}