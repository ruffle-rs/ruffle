/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package testpublicClassWithParamCons{

       public class publicClassWithParamCons{
                private var x:Number=0;
                private const y:Number;
                public function publicClassWithParamCons(a:Number,b:Number){
                                                                           x = a;
                                                                           this.y=b;
                                                                           }
                                              

                                        

                public function Add():Number{
                     var z:Number = x+y;
                     return z;
                                     
                }
                                             

       }
                              
 
       
}
