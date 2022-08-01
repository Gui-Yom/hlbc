class Enums {
    static function main() {
        var a = Red;
        var b = Green;
        var c = Rgb(255, 255, 0);
        switch (c) {
            case Red:
                trace("red");
            case Green:
                trace("Color was green");
            case Blue:
                trace("Color was blue");
            case Rgb(r, g, b):
                trace("Color had a red value of " + r);
        }
    }
}

enum Color {
  Red;
  Green;
  Blue;
  Rgb(r:Int, g:Int, b:Int);
}
