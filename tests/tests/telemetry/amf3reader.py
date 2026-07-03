#!/usr/bin/python 
# Library to parse an AMF3 formatted buffer into native python
# or can be used to convert AMF3 into JSON

# Copyright 2013 Adobe Systems Incorporated.
# 
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0.html 

# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


from struct import unpack
from datetime import datetime

class Metric:
    pass

def ByteToHex( byteStr ):
    return ' '.join( [ "%02X" % (x if isinstance(x, int) else ord(x)) for x in byteStr ] )

class amf3reader(dict):
    """
    Reads AMF3 data
    This can be used with streaming data or with a static buffer
    The data can be passed when creating the instance or passed in pieces using addData
    Read using "read" to get one object at a time, or use "unpack" to unpack the entire buffer
    (in many cases this will amount to the same thing)
    if called on its own this file will convert amf content to python
    """

    # Set verbose to True to get a detailed report on AMF3 encoding with hex
    verbose = False

    kUndefinedAtomType      = 0
    kNullAtomType           = 1
    kFalseAtomType          = 2
    kTrueAtomType           = 3
    kIntegerAtomType        = 4
    kDoubleAtomType         = 5
    kStringAtomType         = 6
    kAvmMinusXmlAtomType    = 7
    kDateAtomType           = 8
    kArrayAtomType          = 9
    kObjectAtomType         = 10
    kAvmPlusXmlAtomType     = 11
    kByteArrayAtomType      = 12
    kTypedVectorIntType     = 13
    kTypedVectorUintType    = 14
    kTypedVectorDoubleType  = 15
    kTypedVectorObjectType  = 16
    kDictionaryObjectType   = 17

    def __init__(self, newData = None):
        # Lock on to the file
        self.data = ""
        if newData:
            self.data = newData
        self.pos = 0        
        self.stringList = []
        self.traitsList = []
        self.objectsList = []
        self.format = None
        self.flash11Mode = False  
        
    def setData(self, data):
        self.data = data;
        self.getFormat()

    def addData(self, data):
        self.data += data;
        self.getFormat()
                        
    def addString(self, string):
        index = len(self.stringList)
        self.stringList.append(string)
        return index

    def addObject(self, obj):
        self.objectsList.append(obj);
        
    def getObject(self, index):
        if index < len(self.objectsList):
            obj = self.objectsList[index];
            return obj;
        else :
            print(("invalid object reference" + str(index)));
            raise Exception("Invalid reference");
        
    def clearObjectsList(self):
        self.objectsList = []
        
    def getString(self,index):
        if index < len(self.stringList):
            string = self.stringList[index];
        else:
            string = "unknown"
            print(("invalid string reference " + str(index)))
        return string
        
    def printHex(self,count):
         if self.verbose and self.pos+count <= len(self.data):
             print(ByteToHex(self.data[self.pos:self.pos+count]), end=' ') 
        
    def peekByte(self):
        if (self.pos < len(self.data)):
            return self.data[self.pos]
        else:
            return None
            
    def readByte(self):
        val = None
        if self.pos+1 <= len(self.data):
            val = self.data[self.pos]
            self.printHex(1)
            self.pos += 1
            return val
        else:
            raise EOFError;
        return val
    
    def readInt(self):
        val = None
        if self.pos+4 <= len(self.data):
            val = unpack('>I', self.data[self.pos:self.pos+4])[0]
            self.pos += 4
        else:
            raise EOFError;
        return val
                

    def readShort(self):
        val = None
        if self.pos+2 <= len(self.data):
            val =  unpack('>H', self.data[self.pos:self.pos+2])[0]
            self.pos += 2;
        else:
            raise EOFError;
        return val
        

    def readDouble(self):
        val = None
        if self.pos+8 <= len(self.data):
            val = unpack('>d', self.data[self.pos:self.pos+8])[0]
            self.printHex(8)
            self.pos += 8
        else:
            raise EOFError;
        return val


    def readBytes(self,length):
        val = None
        if (self.verbose): print("bytes: ", end=' ')
		
        if self.pos+length <= len(self.data):
            val = self.data[self.pos:self.pos+length]
            self.printHex(length)
            self.pos += length
        else:
            raise EOFError;
            
        if (self.verbose): print(""); # Move to next line (self.printHex above would have printed all bytes)
        return val


    def readUint29(self):         
        byte = self.readByte()
        if byte == None: return None
        if byte < 128:
            return byte;        
        ref = (byte & 0x7F) << 7
        byte = self.readByte()
        if byte == None: return None
        if byte < 128:
            return (ref | byte)
        ref = (ref | (byte & 0x7F)) << 7
        byte = self.readByte()
        if byte == None: return None
        if byte < 128:
            return (ref | byte)
        ref = (ref | (byte & 0x7F)) << 8
        byte = self.readByte()
        if byte == None: return None
        return (ref | byte);


    def readAmfString(self, stringWithoutMarker, noCache=False):
        """ reads and AMF formatted string tracking references """
        
        if (self.verbose and stringWithoutMarker):
            print("String(wm) ", end=' ')
        
        ref = self.readUint29()
        
        if ref == None: return None
        if (ref & 1) == 0:
            if (self.verbose) : print("Ref: %d" %(ref>>1), end=' ')
            return self.getString(ref >> 1);
        length = ref >> 1;
        if length == 0:
            return ""
        
        if (self.verbose) : print(" (%d) Len: %d" % (len(self.stringList), (ref>>1)), end=' ')
        s = self.readBytes(length)
        
        # Python 3: decode bytes to string
        if isinstance(s, bytes):
            s = s.decode('utf-8')
        
        if not noCache:  # for flash11 support
            self.addString(s)
        return s

    def readAmfObject(self) :
        encoding = self.readByte();
        if encoding == None:
            return None

        encoding = encoding & 255;
        value = None;

        if encoding == self.kIntegerAtomType:
            value = self.readUint29(); 
            if (self.verbose) : print("Int29=%d" % value);
        elif encoding == self.kDoubleAtomType:
            value = self.readDouble(); 
            if (self.verbose) : print("Double=%g" % value);
        elif encoding == self.kStringAtomType:
            if (self.verbose) : print("String ", end=' ')
            value = self.readAmfString(False, self.flash11Mode); # early format support
            if (self.verbose) : print(" \"%s\"" % value);
        elif encoding == self.kNullAtomType:
            value = None;
            if (self.verbose) : print("Null")
        elif encoding == self.kUndefinedAtomType:
            value = None;
            if (self.verbose) : print("Undefined")
        elif encoding == self.kFalseAtomType:
            value = False; 
            if (self.verbose) : print("False")
        elif encoding == self.kTrueAtomType:
            value = True; 
            if (self.verbose) : print("True")
        elif encoding == self.kDateAtomType:
            ref = self.readUint29()
            
            if (self.verbose) : print("Date ", end=' ')
            
            if (ref & 1) == 0:
                value = self.getObject(ref>>1);
                if (self.verbose): print("Ref: " + str(ref>>1));
            else:
                if (self.verbose) : print("(%d)" % len(self.objectsList));
                value = self.readDouble(); # dates are written as doubles
                if (self.verbose) : print(" " + str(value) + " " + datetime.fromtimestamp(value/1000).isoformat());
                self.addObject(value);
        elif (encoding == self.kAvmMinusXmlAtomType or
            encoding == self.kAvmPlusXmlAtomType ):
            ref = self.readUint29()
            if (self.verbose) : 
                print("XML ", end=' ')
                if encoding == self.kAvmMinusXmlAtomType :
                    print("- ", end=' ')
                else :
                    print("+ ", end=' ')
            
            if (ref & 1) == 0:
                value = self.getObject(ref>>1);
                if (self.verbose) : print("Ref: " + str(ref>>1));
            else:
                if (self.verbose) : print("(%d)" % len(self.objectsList));
                if (self.verbose) : self.printHex(ref >> 1)
                value = self.readBytes(ref >> 1); # return as string for now
                if (self.verbose) : print(value.encode())
                self.addObject(value);
        elif encoding == self.kDictionaryObjectType:
            ref = self.readUint29()
            
            if (self.verbose) : print("Dictionary ", end=' ')
            if (ref & 1) == 0:
                value = self.getObject(ref>>1);
                if (self.verbose) : print("Ref: " + str(ref>>1));
            else:
                if (self.verbose) : 
                    print("(%d)" % len(self.objectsList), end=' ')
                    print(" count: %d" %(ref>>1));
                weakref = self.readByte() == 1
                if (self.verbose) : print("weakRef");
                count = ref >> 1
                value = {}
                self.addObject(value);
                while count > 0:
                    key = self.readAmfObject()
                    val = self.readAmfObject()
                    #print key
                    #print val
                    value[str(key)] = val
                    count -= 1
                if (self.verbose) : print('\n', end=' ')
        elif encoding == self.kArrayAtomType:
            value = {}
            ref = self.readUint29()
            if (self.verbose) : print("Array ", end=' ')
            
            if (ref & 1) == 0 :
                value = self.getObject(ref>>1);
                if (self.verbose) : print("Ref: " + str(ref));
            else :
                if (self.verbose) : 
                    print("(%d)" % len(self.objectsList))
                    print("count: %d" % (ref>>1));
                self.addObject(value);
                count = ref >> 1
                # read the non-dense portion
                s = self.readAmfString(True)
                while  s and len(s)>0:
                    if (self.verbose) : print("%s (dyn)" % (s))
                    v = self.readAmfObject()
                    value[s] = v
                    s = self.readAmfString(True)
                    if (self.verbose) : print("")
                 #now read the dense portion
                i = 0
                while i < count:
                    if (self.verbose) : print("[%d]" % i)
                    value[i] = self.readAmfObject();
                    i += 1
                if (self.verbose) : print('\n', end=' ')
        elif ( encoding == self.kTypedVectorIntType or
            encoding == self.kTypedVectorUintType or
            encoding == self.kTypedVectorDoubleType or
            encoding == self.kTypedVectorObjectType ):
            ref = self.readUint29()
            
            if (self.verbose) : 
                if (encoding == self.kTypedVectorIntType):
                    print("Vector Int", end=' ')
                elif (encoding == self.kTypedVectorUintType) :
                    print("Vector Uint", end=' ')
                elif (encoding == self.kTypedVectorDoubleType) :
                    print("Vector Double", end=' ')
                else:
                    print("Vector Object", end=' ')
            
            if (ref & 1) == 0:
                value = self.getObject(ref>>1);
                print(" Ref: " + str(ref>>1));
            else:
                count = ref >> 1
                if (self.verbose): print("length = %d" % count)
                fixed = self.readByte() == 1
                if (self.verbose) : print("fixed")
                value = []
                self.addObject(value);
                if (encoding == self.kTypedVectorIntType or encoding == self.kTypedVectorUintType):
                    while(count > 0):
                        self.printHex(4)
                        value.append(self.readInt())
                        count -= 1
                        if (self.verbose) : print(" ")
                    if (self.verbose): print(value)
                elif encoding == self.kTypedVectorDoubleType:
                    while(count > 0):
                        value.append(self.readDouble())
                        count -= 1
                elif encoding == self.kTypedVectorObjectType:
                    className = self.readAmfString(True);
                    if (self.verbose): print("ClassName = %s" % className)
                    while (count > 0):
                        value.append(self.readAmfObject());
                        count -= 1
                if (self.verbose) : print('\n', end=' ')
        elif encoding == self.kByteArrayAtomType:
            ref = self.readUint29();
            
            if (self.verbose):
                print("ByteArray ", end=' ')
            
            if (ref & 1) == 0:
                value = self.getObject(ref>>1);
                if (self.verbose) : print("Ref: " + str(ref>>1))
            else:
                if (self.verbose) : print("count: " + str(ref>>1));
                value = self.readBytes(ref>>1);
                self.addObject(value);
        elif encoding == self.kObjectAtomType:
            ref = self.readUint29()
            #if ref == None: return None
            if (self.verbose):
                print("Object ", end=' ')
            if (ref & 1) == 0: 
                value = self.getObject(ref>>1);
                if (self.verbose) : print("Ref: " + str(ref>>1));
            else :
                if (self.verbose) : print("(%d)" % len(self.objectsList), end=' ')
                if ((ref & 3) == 1):
                    if (self.verbose) : print("Traits Ref: " + str(ref>>2) + " (class: %s slots: %d dynamic: %d)" %(self.traitsList[ref>>2]['className'], len(self.traitsList[ref>>2]['slots']), self.traitsList[ref>>2]['dynamic']));
                    traits = self.traitsList[ref >> 2];
                else:
                    traits = {}
                    traits['dynamic'] = ((ref & 8) >> 3);

                    if ref & 4:
                        traits['externalizable'] = True
                        
                    traits['count'] = ref >> 4
                    
                    if self.verbose: 
                         print("Traits (%d) slots: %d dynamic: %d" % (len(self.traitsList), (ref>>4), ((ref & 8) >> 3)))
                    
                    className = self.readAmfString(True)
                    
                    if self.verbose:
                        print("class: " + className)
                    
                    if className and len(className) > 0:
                        traits['className'] = className
                    else : 
                        traits['className'] = ""
                    #new_class = type(className', (object,), {));
                    slots = [];
                    count = traits['count']
                    while (count):
                        slots.append(self.readAmfString(True))
                        if self.verbose: print(slots[len(slots)-1])
                        count -= 1
                    traits['slots'] = slots
                    self.traitsList.append(traits)
                value = {}
                
                self.addObject(value);
                
                #if  traits.has_key('className'):
                #    value['_className'] = traits['className']
                for slot in traits['slots']:
                    value[slot] = self.readAmfObject()
                    
                if (traits['dynamic'] == 1):
                    s = self.readAmfString(True)
                    while  s and len(s)>0:
                        if (self.verbose) : print("%s (dyn)" % (s))
                        v = self.readAmfObject()
                        value[s] = v
                        s = self.readAmfString(True)
                if (self.verbose) : print('\n', end=' ')
        else:
            print(("invalid data type", encoding));
            #throw new Error("invalid data stream");
            value = None;
        return value;

    # Examine data stream to find its format
    def getFormat(self):
        if (len(self.data)<1):
            return None
        firstByte = self.data[0]
        if firstByte == self.kObjectAtomType:
            self.format = "amfstream"   # raw stream from player
        elif firstByte == self.kArrayAtomType:
            self.format = "amfarray"  # saved array of telemetry data
        else:
            self.format = "oldstyle"  # early format
            self.flash11Mode = True
        return self.format
            
    
    def readMetric(self):
        recordPos = self.pos; 
        record = None;
        if self.flash11Mode:
            # this is to support older telemetry, remove eventually
            try:
                name = self.readAmfString(True)
            
                if (self.verbose) : print("String=%s" % name);
                if name.endswith(".span"):
                    name = name.rsplit('.',1)[0] # strip the .span extension
                    span = self.readAmfObject();
                    tname = self.readAmfString(True);
                    if (self.verbose) : print("String=%s" % tname);
                    time = self.readAmfObject();
                    if name is not None and tname is not None and tname.endswith(".time") \
                        and span is not None and time is not None:
                        record = {'name':name,"span":span,"time":time};
                elif name.endswith(".time"):
                    name = name.rsplit('.',1)[0] # strip unused types
                    time = self.readAmfObject()
                    if time:
                        record = {'name':name,'time':time}
                else:
                    if name.endswith(".count"):
                        name = name.rsplit('.',1)[0] # strip unused types
                    record = {'name':name}
                    record['value'] = self.readAmfObject();
            except:
                self.pos = recordPos; #rewind to start
                name = None
        else:    
            traitsLen = len(self.traitsList)
            stringCount = len(self.stringList) 
            try:
                record = self.readAmfObject();
            except EOFError:
                if (self.verbose):
                    print("Partial record warning")
                self.pos = recordPos; #rewind to start
                self.traitsList = self.traitsList[0:traitsLen];
                self.stringList = self.stringList[0:stringCount];
                record = None;
            finally:
                self.clearObjectsList();
        return record


    # reads the entire buffer as one array of objects
    def unpack(self):
        output = [] # all the data ends up in this array
        rec = self.readMetric()
        while (rec and self.pos+1 < len(self.data)):
            output.append(rec)
            rec = self.readMetric()

        if (rec):
            output.append(rec);
        return output 



if __name__ == '__main__':
    import sys
    from pprint import pprint

    if len(sys.argv) == 1:
        print('Usage: %s filename [filename]...' % sys.argv[0])
        print('Where filename is a .flm file')
        print('eg. %s myfile' % sys.argv[0])
    for filename in sys.argv[1:]:
        file = open(filename, 'rb')
        reader = amf3reader(file.read());
        file.close()
        output = reader.unpack()
        pprint(output) # print the array to stdout in JSON format
