/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;

// package {

//     var SECTION = "15.2";
//     var VERSION = "ECMA_3";
//     var TITLE   = "JSON ecma-262 testcases";



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


    Assert.expectEq("15.12-0-1: JSON must be a built-in object","object",typeof(JSON));

    // 15.12-0-2: JSON must not support the [[Construct]] method
    var constructorException="no exception";
    try {
        var j = new JSON();
    } catch(e) {
        constructorException=e.toString();
        constructorException=removeExceptionDetail(constructorException);
    }
    Assert.expectEq("15.12-0-2: JSON must not support the [[Construct]] method","ArgumentError: Error #2012",constructorException);

    // 15.12-0-3: JSON must not support the [[Call] method
    var callException="no exception";
    try {
        var k = JSON();
    } catch(e) {
        callException=e.toString();
        callException=removeExceptionDetail(callException);
    }
    Assert.expectEq("15.12-0-3: JSON must not support the [[Call]] method","ArgumentError: Error #1112",callException);

    // 15.12-0-4: JSON object properties must be non enumerable
    var i=0;
    for (var p in JSON) {
        i++;
    }
    Assert.expectEq("15.12-0-4: JSON object properties must be non enumerable",0,i);


//}
