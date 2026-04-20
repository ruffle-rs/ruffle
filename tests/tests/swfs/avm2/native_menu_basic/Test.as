package {
    import flash.display.MovieClip;
    import flash.display.NativeMenu;
    import flash.display.NativeMenuItem;
    import flash.text.TextField;

    public class Test extends MovieClip {
        private var outputField:TextField;

        public function Test() {
            super();
            this.outputField = new TextField();
            this.outputField.width = 400;
            this.outputField.height = 400;
            this.addChild(this.outputField);

            try {
                var menu:NativeMenu = new NativeMenu();
                output(menu.parent);
                output(menu.items == menu.items);
                menu.items.push(3);
                output(menu.items);
                try {
                    menu.items = [new NativeMenuItem("test0"), 4, new NativeMenuItem("test1")];
                } catch(e:Error) {
                    output(Object.prototype.toString.call(e));
                    output(e.errorID);
                }
                output(menu.items);
                menu.items = [];
                output(menu.items);
                menu.items = [new NativeMenuItem("test2"), new NativeMenuItem("test3")];
                output(menu.items);
                try {
                    menu.addItem(null);
                } catch(e:Error) {
                    output(Object.prototype.toString.call(e));
                    output(e.errorID);
                }
                output(menu.items);
                menu.addItem(new NativeMenuItem("test4"));
                output(menu.items);
                try {
                    menu.addItemAt(new NativeMenuItem("test5"), -1);
                } catch(e:Error) {
                    output(Object.prototype.toString.call(e));
                    output(e.errorID);
                }
                output(menu.items);
                menu.addItemAt(new NativeMenuItem("test6"), 3);
                output(menu.items);
                try {
                    menu.addItemAt(new NativeMenuItem("test5"), 5);
                } catch(e:Error) {
                    output(Object.prototype.toString.call(e));
                    output(e.errorID);
                }
                output(menu.items);
                menu.removeAllItems();
                output(menu.items);
            } catch(e:Error) {
                output(e);
            }
        }
        
        function output(info:*):void {
            var result:String = "";
            if (info is Array) {
                for (var i:int = 0; i < info.length; i ++) {
                    if (info[i] is NativeMenuItem) {
                        result += "[NMI](" + info[i].label + ")";
                    } else {
                        result += info[i];
                    }
                    if (i != info.length - 1) {
                        result += ",";
                    }
                }
            } else if (info is NativeMenuItem) {
                result = "[NMI](" + info.label + ")";
            } else {
                result = info;
            }
            trace(result);
            this.outputField.text += result + "\n";
        }
    }
}
