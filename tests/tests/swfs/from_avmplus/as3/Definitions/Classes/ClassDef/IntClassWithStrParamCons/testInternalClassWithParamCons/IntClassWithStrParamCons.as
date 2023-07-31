/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
package testInternalClassWithParamCons{

 internal class IntClassWithStrParamCons{
             public var x:String;
             public var myarr:Array;
             public var myObj:publicClassCons;
              
  function IntClassWithStrParamCons(a:String,b:Boolean,c:Array,d:publicClassCons,e:DefaultClass)

                                                                   {
                                                                     x=a;
                                                                     y=b;
                                                                     myarr=c;
                                                                     myObj=d;
                                                                            
                                                                                }
                                        

                                        public function myString():String{
                                                              
                                                              return x;
                                                                         }
                                        public function myBoolean():Boolean{
                                                              
                                                              return y;
                                                                           }
                                        public function myarray():Array{
                                                             
                                                             return myarr;
                                                                       }
                                        
                                        public function myAdd():Number{
                                                              return myObj.Add();
                                                                      }


                                          }

                         
                                                        }
