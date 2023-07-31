/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package testdynfinalClassWithMultiStringParamCons{

 dynamic final public  class dynfinClassWithMultiStrParamCons{
             public var x:String;
             
              
   public function dynfinClassWithMultiStrParamCons(z:String){
                                                            x=z;
                                                        }
                                        

                   public function myString():String{
                                                              
                                                     return x;
                                                     }
                  public function myString2():String{
                  var x:String="test2";
                  var y:String="test3";
                  var g1= new dynfinClassWithMultiStrParamCons("test2");
                  var g3=new dynfinClassWithMultiStrParamCons("hello"+
"world");
                                    
                                     return g3.myString();


                                          }
  
                                   
}


}




