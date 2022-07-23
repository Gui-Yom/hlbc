class Switch {
    static function main() {
        var a = 3;
        var b = switch (a) {
            case 0: a * 2;
            case 3: a - 1;
            default: a << 2;
        }
        switch (b) {
            case 0: b += 1;
            case 1: b += 1;
            case 2: b += 1;
            //case 4: b += 1;
        }
        /*
        switch (a) {
            case -1: b += 1;
            case -5: b += 1;
        }
        var c = "hello";
        switch (c) {
            case "hello": a += 1;
            case "world": a += 1;
        }*/
    }
}