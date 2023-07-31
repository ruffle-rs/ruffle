/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package publicClassConstructors{
public class publicClassDefCons{
private var x:Number=10;
private var y:Number=20;
private var s:Boolean=true;
private var mydatatype:String="I am a string";

final public function Add():Number{
                                   var z:Number=x+y;
                                   return z;
                                  }
private function changeval():Boolean{
                                     return (!s);
                                    }
public function currentdate():Date{
                                   var today = new Date(0);
                                   return today;
                                   }

internal function myobject():Object{
                                    return mydatatype;
                                   }
//wrapper function for internal myobject function
public function wrapintmyobject():Object{
                                         return myobject();
                                        }
//wrapper function for private function changeval
public function wrapprivchangeval():Boolean{
return changeval();
}

public static function main():void{
var DefCons:publicClassDefCons = new publicClassDefCons();
//print("I am from the main method"+ DefCons.Add());
}
}
}
