/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package testInternalClassWithParamCons{

public class wrapIntClassWithStrParamCons{
   var x = "test";
   var y:Boolean = true;
   var myArray:Array = new Array(4,6,5);
   var pbClCons:publicClassCons = new publicClassCons();
   var MyDefaultClass:DefaultClass;
   var ICWSPS:IntClassWithStrParamCons = new IntClassWithStrParamCons(x,y,myArray,pbClCons,MyDefaultClass);
             public function myArray1():Array{
                                             return myArray;
                                             }

             public function wrapmyString():String{
                                               var w:String = ICWSPS.myString();
                                                              
                                               return w;
                                                }
             public function wrapmyBoolean():Boolean{
                                                var H:Boolean = ICWSPS.myBoolean();
                                                return H;
                                                }
                                        
             public function wrapmyarray():Array{
                                                var I:Array = ICWSPS.myarray();
                                                return I;
                                                 }
                                        
              public function wrapmyAdd():Number{
                                                var J:Number = ICWSPS.myAdd();
                                            return J;
                                            }

                                         }


                         
                                                        }
