.flash bbox=300x200 version=8 name="test.swf" compress
.action:
    var counter = 0;
    function traceParseInt() {
        var args;
        var result;
        switch (arguments.length) {
        case 0:
            args = '';
            result = parseInt();
            break;
        case 1:
            args = "'" + arguments[0].toString() + "'";
            result = parseInt(arguments[0]);
            break;
        case 2:
            args = "'" + arguments[0].toString() + "', " + arguments[1].toString();
            result = parseInt(arguments[0], arguments[1]);
            break;
        }
        trace('/*' + counter++ + '*/ parseInt(' + args + ') == ' + result.toString());
    }

    var undefined_;

    traceParseInt();
    traceParseInt(undefined_);
    traceParseInt(undefined_, 32);
    traceParseInt('');
    traceParseInt('123');

    traceParseInt('100', 10);
    traceParseInt('100', 0);
    traceParseInt('100', 1);
    traceParseInt('100', 2);
    traceParseInt('100', 36);
    traceParseInt('100', 37);
    traceParseInt('100', -1);
    traceParseInt('100', {});
    traceParseInt('100', true);
    traceParseInt('100', false);
    traceParseInt('100', NaN);
    traceParseInt('100', undefined_);

    traceParseInt('0x123');
    traceParseInt('0xabc');
    traceParseInt('010', 2);
    traceParseInt('-0100');
    traceParseInt('-0100z');
    traceParseInt('0x+0X100');
    traceParseInt(123);
    traceParseInt(123, 32);
    traceParseInt('++1');

    traceParseInt('0x100', 36);
    traceParseInt(' 0x100', 36);
    traceParseInt('0y100', 36);
    traceParseInt(' 0y100', 36);

    traceParseInt('-0x100', 36);
    traceParseInt(' -0x100', 36);
    traceParseInt('-0y100', 36);
    traceParseInt(' -0y100', 36);

    traceParseInt('-0x100');
    traceParseInt('0x-100');
    traceParseInt(' 0x-100');
    traceParseInt('0x -100');
    traceParseInt('-0100');
    traceParseInt('0-100');

    traceParseInt('+0x123', 33);
    traceParseInt('+0x123', 34);

    traceParseInt('0');
    traceParseInt(' 0');
    traceParseInt(' 0 ');

    traceParseInt('077');
    traceParseInt('  077');
    traceParseInt('  077   ');
    traceParseInt('  -077');
    traceParseInt('077 ');

    traceParseInt('11', 2);
    traceParseInt('11', 3);
    traceParseInt('11', 3.8);

    traceParseInt('0x12');
    traceParseInt('0x12', 16);
    traceParseInt('0x12', 16.1);
    traceParseInt('0x12', NaN);
    traceParseInt('0x  ');
    traceParseInt('0x');
    traceParseInt('0x  ', 16);
    traceParseInt('0x', 16);
    traceParseInt('12aaa');

    traceParseInt('100000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '000000000000000');

    traceParseInt('0x1000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '00000000000000000000000000000000000000000000000000000000000000000000'
        + '000000000000000');
.end
.end
