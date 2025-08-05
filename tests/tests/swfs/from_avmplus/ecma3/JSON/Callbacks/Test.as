/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

//package {
//     var SECTION = "15.2";
//     var VERSION = "ECMA_5";
//     var TITLE   = "JSON AS3 Callback tests";



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


    Assert.expectEq('simple reviver',true,JSON.parse('[1,2,3]',function f() { return true; }));

    var keys=[];
    JSON.parse('[1,2,3]',function f(key,value) { keys[keys.length]=key; });
    keys.sort();
    Assert.expectEq('simple reviver test keys',',0,1,2',keys.toString());

    var values=[];
    JSON.parse('[1,2,3]',function f(key,value) { values[values.length]=value; });
    values.sort();
    Assert.expectEq('simple reviver test values',",,,1,2,3",values.toString());

    var keys2=[];
    JSON.parse('{"a":1,"b":2,"c":3}',function f(key,value) { keys2[keys2.length]=key; });
    keys2.sort();
    Assert.expectEq('simple reviver test keys on Object',',a,b,c',keys2.toString());

    var values2=[];
    JSON.parse('{"a":1,"b":2,"c":3}',function f(key,value) { values2[values2.length]=value; });
    values2.sort();
    Assert.expectEq('simple reviver test values on Object',"1,2,3,[object Object]",values2.toString());

    Assert.expectEq('simple reviver replaces values','1,0,0,4,5',JSON.parse('[1,2,3,4,5]',function f(key,value) { if (value=='2' || value=='3') return 0; else return value; }).toString());

    Assert.expectEq("simple reviver undefined removes values",'{"c":3,"d":4}',sortObject(JSON.parse('{"a":1,"b":2,"c":3,"d":4}',function f(key,value) { if (key=='a' || key=='b') return undefined; else return value; })));

//}
