/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}


    import avmplus.*;
    import flash.utils.*;
import com.adobe.test.Assert;

//     var SECTION = "15.2";
//     var VERSION = "ECMA_5";
//     var TITLE   = "JSON AS3 Classs types";


    class testclass1 {
        public var var1;
        public var var2:Boolean;
        public function testclass1() {
            var1="var1_value";
            var2=false;
            var3="var3_value";
            var4="var4_value";
            transient1="transient_value1";
        }
        public function foo() {
        }
        private var var3;
        private var var4:String;

        public function get getonly() {
            return "getonly_value";
        }

        public function set setonly(s) {
        }

        [Transient]
        public var transient1;

        [Transient]
        [ExcludeClass]
        public function get transientgetter() {
            return "transient getter";
        }
    }

    class testclass2 {
        public var var1:int=-1;
        public var var2:Number=-3.14;
        public var var3:Array=[1,2,3];
        public var var4:Object=new Object();
        public var var5=Infinity;
        function testclass2() {
            var4.prop1=10;
        }
    }



function removeExceptionDetail(s:String) {
    var fnd=s.indexOf(" ");
    if (fnd>-1) {
        if (s.indexOf(':',fnd)>-1) {
                s=s.substring(0,s.indexOf(':',fnd));
        }
    }
    return s;
}

function sortObject(o:Object) {
    var keys=[];
    var key;
    for ( key in o ) {
        if (o[key]===undefined) {
           continue;
        }
        keys[keys.length]=key;
    }
    keys.sort();
    var ret="{";
    var value;
    for (var i in keys) {
        key=keys[i];
        value=o[key];
        if (value is String) {
            value='"'+value+'"';
        } else if (value is Array) {
            value='['+value+']';
        } else if (value is Object) {
        }
        ret += '"'+key+'":'+value+",";
    }
    ret=ret.substring(0,ret.length-1);
    ret+="}";
    return ret;
}


    Assert.expectEq("stringify an AS3 class with getters, transient properties",'{"getonly":"getonly_value","var1":"var1_value","var2":false}',sortObject(JSON.parse(JSON.stringify(new testclass1()))));

    Assert.expectEq("stringify an AS3 class",'{"var1":-1,"var2":-3.14,"var3":[1,2,3],"var4":[object Object],"var5":null}',sortObject(JSON.parse(JSON.stringify(new testclass2()))));

    var testobj:Object=new Object();
    testobj.prop1="1";
    testobj.prop2=true;
    testobj.prop3=undefined;
    testobj.prop4=null;
    testobj.prop5=10;
    testobj.prop6=10.11;
    testobj.prop7=NaN;
    testobj.prop8=-Infinity;

   Assert.expectEq("stringify an AS3 object with various methods",'{"prop1":"1","prop2":true,"prop4":null,"prop5":10,"prop6":10.11,"prop7":null,"prop8":null}',sortObject(JSON.parse(JSON.stringify(testobj))));

    var testobj2:Object=new Object();
    testobj2.prop1=10;
    testobj2.toJSON=function() {
        return "testobj2";
    }
    Assert.expectEq("stringify an AS3 object with toJSON","\"testobj2\"",JSON.stringify(testobj2));

    var testobj3:Object=new Object();
    testobj3.prop1=10;
    testobj3.toJSON=function f() {
        throw new Error("toJSON error");
    }
    var exception1="no exception";
    try {
        JSON.stringify(testobj3);
    } catch (e) {
        exception1=e.toString();
    }
    Assert.expectEq("stringify an AS3 object with toJSON","Error: toJSON error",exception1);

    var testobj4:Object=new Object();
    testobj4.toJSON=true;
    Assert.expectEq("stringify an AS3 object with non-function toJSON",'{"toJSON":true}',JSON.stringify(testobj4));


