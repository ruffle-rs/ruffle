package {
import flash.display.MovieClip;
import flash.text.TextField;
import flash.display.SimpleButton;
import flash.display.Sprite;
import flash.display.MovieClip;
public class Test extends MovieClip {
    public function Test() {
        super();

        var text:TextField = new TextField();
        var text2:TextField = new TextField();
        text2.type = "input";
        var button:SimpleButton = new SimpleButton();
        var mc:MovieClip = new MovieClip();
        var mc2:MovieClip = new MovieClip();
        mc2.buttonMode = true;
        var mc3:MovieClip = new MovieClip();
        var sprite:Sprite = new Sprite();

        trace('===== stage =====');
        this.testProperties(this.stage);
        trace('===== text =====');
        this.testProperties(text);
        trace('===== text type input =====');
        this.testProperties(text2);
        trace('===== button =====');
        this.testProperties(button);
        trace('===== movie clip =====');
        this.testProperties(mc);
        trace('===== movie clip button mode true =====');
        this.testProperties(mc2);
        trace('===== movie clip with index =====');
        mc3.tabIndex = 4;
        this.testProperties(mc3);
        trace('===== sprite =====');
        this.testProperties(sprite);
    }

    function logError(f:*):void {
        try {
            f();
        } catch (error:Error) {
            trace('    Error: ' + error);
        }
    }

    function printProperties(obj:*):void {
        this.logError(function():void {
            trace('    tabEnabled = ' + obj.tabEnabled);
        });
        this.logError(function():void {
            trace('    tabIndex = ' + obj.tabIndex);
        });
        this.logError(function():void {
            trace('    tabChildren = ' + obj.tabChildren);
        });
        for (var i:String in obj) {
            if (i == 'tabEnabled') {
                trace('    enumerated tabEnabled');
            }
        }
        for (i in obj) {
            if (i == 'tabIndex') {
                trace('    enumerated tabIndex');
            }
        }
        for (i in obj) {
            if (i == 'tabChildren') {
                trace('    enumerated tabChildren');
            }
        }
    }

    function testProperties(obj:*):void {
        trace('  default');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = true;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = 0;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = true;
        });

        trace('  after set 1');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = false;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = 4;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = false;
        });

        trace('  after set 2');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = undefined;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = undefined;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = undefined;
        });

        trace('  after set 3');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = -4;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = -4;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = -4;
        });

        trace('  after set 4');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = 2147483647;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = 2147483647;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = 2147483647;
        });

        trace('  after set 5');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = 2147483648;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = 2147483648;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = 2147483648;
        });

        trace('  after set 6');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = 'x';
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = 'x';
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = 'x';
        });

        trace('  after set 7');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = -2147483648;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = -2147483648;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = -2147483648;
        });

        trace('  after set 8');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = new Object();
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = new Object();
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = new Object();
        });

        trace('  after set 9');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = 1.1;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = 1.1;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = 1.1;
        });

        trace('  after set 10');
        this.printProperties(obj);

        this.logError(function():void {
            trace('    set tabEnabled');
            obj.tabEnabled = -1;
        });
        this.logError(function():void {
            trace('    set tabIndex');
            obj.tabIndex = -1;
        });
        this.logError(function():void {
            trace('    set tabChildren');
            obj.tabChildren = -1;
        });

        trace('  after set 11');
        this.printProperties(obj);
    }
}
}
