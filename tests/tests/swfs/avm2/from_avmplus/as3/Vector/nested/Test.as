/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

package {
import flash.display.MovieClip;
public class Test extends MovieClip {}
}

import com.adobe.test.Assert;
/**
 File Name:    nested.as
 Description:  tests nested Vectors: Vector.<Vector.<int>>
 *
 */
// var SECTION="";
// var VERSION = "ECMA_1";



var v1:Vector.<Vector.<int>>=new Vector.<Vector.<int>>();
var v2:Vector.<int>;
v2=new Vector.<int>;v2[0]=0;v2[1]=1;
v1.push(v2);
v2=new Vector.<int>;v2[0]=2;v2[1]=3;
v1.push(v2);
v2=new Vector.<int>;v2[0]=4;v2[1]=5;
v1.push(v2);
Assert.expectEq("push nested vector.<int>",
  "0,1,2,3,4,5",
  v1.toString());

var v3:Vector.<Vector.<String>>=new Vector.<Vector.<String>>();
var v4:Vector.<String>;
v4=new Vector.<String>;v4[0]='one';v4[1]='two';
v3.push(v4);
v4=new Vector.<String>;v4[0]='three';v4[1]='four';
v3.push(v4);
v4=new Vector.<String>;v4[0]='five';v4[1]='six';
v3.push(v4);
Assert.expectEq("push nested vector.<String>",
  "one,two,three,four,five,six",
  v3.toString());

var v5:Vector.<Vector.<Vector.<int>>>=new Vector.<Vector.<Vector.<int>>>();
var v6:Vector.<Vector.<int>>;
var v7:Vector.<int>;
v7=new Vector.<int>();v7[0]=0;v7[1]=1;
v6=new Vector.<Vector.<int>>();
v6.push(v7);
v5.push(v6);
v7=new Vector.<int>();v7[0]=2;v7[1]=3;
v6=new Vector.<Vector.<int>>();
v6.push(v7);
v7=new Vector.<int>();v7[0]=4;v7[1]=5;
v6.push(v7);
v5.push(v6);

Assert.expectEq("push nested vector.<vector.<int>>",
  "0,1,2,3,4,5",
  v5.toString());

class tree {
  var value:String;
  var left:tree;
  var right:tree;
  function tree(value:String,left:tree,right:tree){
    this.value=value;
    this.left=left;
    this.right=right;
  }
  static function depthfirst(t:tree):String {
    var out="";
    out+=t.value;
    if (t.left!=undefined) out+=","+depthfirst(t.left);
    if (t.right!=undefined) out+=","+depthfirst(t.right);
    return out;
  }
  static function collect(t:tree):Vector.<tree> {
    var out:Vector.<tree>=new Vector.<tree>();
    out.push(t);
    if (t.left!=undefined) out=out.concat(collect(t.left));
    if (t.right!=undefined) out=out.concat(collect(t.right));
    return out;
  }
  static function printlist(l:Vector.<tree>):String {
    var ret:String="";
    for (var i:int=0;i<l.length;i++) {
      ret+=l[i].value;
      if (i<l.length-1) ret+=",";
    }
    return ret;
  }
}
var two=new tree("two",undefined,undefined);
var three=new tree("three",undefined,undefined);
var one=new tree("one",two,three);
var six=new tree("six",undefined,undefined);
var seven=new tree("seven",undefined,undefined);
var five=new tree("five",six,seven);
var four=new tree("four",five,undefined);
var root=new tree("root",one,four);

Assert.expectEq("test vector of custom classes",
  tree.printlist(tree.collect(root)).toString(),
  "root,one,two,three,four,five,six,seven");

class TestClass {
  private var myVal:Object;
  public function TestClass(v:Object):void {
    myVal = v;
  }
  public function toString():String {
    return myVal.toString();
  }
}

// nested vector stress test
var nestedVectorType = Vector.<TestClass>;
var tempNestedVector = nestedVectorType([new TestClass(2), new TestClass(new Object()), new TestClass("hello")])
var expectedStr = "";

for (var i=0; i<500; i++) {
  nestedVectorType = Vector.<nestedVectorType>;
  tempNestedVector = nestedVectorType([tempNestedVector]);
  expectedStr += ">";
}

Assert.expectEq("Nested vector typecheck",
  true,
  tempNestedVector is nestedVectorType
);

Assert.expectEq("Nested vector length",
  1,
  tempNestedVector.length
);

// last char from nestedVectorType will be ]
expectedStr += "]";

// Compare the end of the nestedVectorType.toString
Assert.expectEq("500 Nested vectors",
  true,
  expectedStr == nestedVectorType.toString().substr(-501)
);

var deeplyNestedVector = new nestedVectorType();

Assert.expectEq("Instantiate deeply nested vector",
  0,
  deeplyNestedVector.length
);
