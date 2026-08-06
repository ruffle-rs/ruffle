#!/usr/bin/python
# Tool to parse flash telemetry content and generate reports
#
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

import sys
from datetime import datetime, timedelta
from time import gmtime, strftime, ctime
from operator import itemgetter
import locale
locale.setlocale(locale.LC_ALL,'')
from optparse import OptionParser
import bisect

import amf3reader


kSwfFrameMarker = '.swf.frame'

def timeStr(time):
    if time is not None:
        delta = timedelta(microseconds=time)
        return str(delta)
    return ''        


class SortDict(dict):       
    """ helper class adds to dict entry and sorts by value descending
    """
    def addTo(self,key,value):
       if key in self:
           self[key] += value
       else:
           self[key] = value
           
    def addMax(self, key, value):
        if value > self.get(key,0):
            self[key] = value
            
    def getSorted(self):
        return sorted(list(self.items()),key=itemgetter(1),reverse=True)
        
    def pprint(self):
        for a in self.getSorted():
            print(a[0],":",locale.format("%d", a[1], grouping=True))
    
    def total(self):
        total = 0
        for k,v in list(self.items()):
            total += v;
        return total
   
            
class Reporter():
    """ Generates summary reports
    """
    
    def __init__(self, metrics=None):
        self.categories = SortDict()
        self.metrics = SortDict()
        self.memory = SortDict()
        self.memCount = SortDict()
        self.memMax = SortDict()
        self.interval = 0
        self.span = 0.0
        self.startTime = 0
        self.endTime = 0
        self.metricThreshold = 0
        if metrics:
            for m in metrics:
                self.addSpan(m)
                
    def getInterval(self):
        """ duration of wall clock time """
        return self.endTime-self.startTime
                
    def getSpan(self):
        """ duration of time inside metrics """
        return self.span
 
    def addSpan(self,metric):  
       name = metric['name']
       if 'span' in metric:
            span = metric['span']
            time = metric['time']
            if self.startTime == 0:
               self.startTime = time
            self.endTime = time
            category = getCategory(name)
            self.categories.addTo(category, span)
            self.metrics.addTo(name, span)
            self.span += span
       elif options.showMemory and getCategory(name)=='Memory':
            value = metric['value']
            #print 'memory', name, value
            self.memMax.addMax(name, value)
            self.memory.addTo(name,value)
            self.memCount.addTo(name, 1) # keep track of count as well
            #self.memory[name] = value
            
    
    def percentSpan(self,value):
        if value:
            return int(round(100/(self.span/value)))
        else:
            return 0
            
    def report(self):
        print("Most time by Category:")
        sortedMetrics = self.metrics.getSorted();
        for a in self.categories.getSorted():
            category = a[0]
            span = a[1]*1.0
            percent = self.percentSpan(span)
            if percent > 0:
                print("%s: %.3f %d%%"  % (category,span/1000, percent))  
                #print category,":",locale.format("%d", span, grouping=True), str(percent)+"%"  
                if options.metrics:
                    for m in sortedMetrics:
                        if getCategory(m[0])==category:
                            if m[1]:
                                percentOfCategory = int(round(100/(span/m[1])))
                            else:
                                percentOfCategory = 0
                            if percentOfCategory > self.metricThreshold:
                                print("  %s: %.3f %d%%"  % ( m[0], m[1]/1000.0, percentOfCategory))
                         
        
        if options.showMemory:                                 
           print("Memory Average:")
           for m in self.memory.getSorted():
               mType = m[0]
               mValue = m[1]
               mCount = self.memCount[mType]
               print("  %s: avg=%d kb, max=%d kb" % (mType, int(mValue/mCount), self.memMax.get(mType,0)))
        #print "Most time by Metrics"
        #for a in sortedMetrics:
        #    percent = self.percentSpan(a[1])
        #   if percent > 0:
        #        print a[0],":",locale.format("%d", a[1], grouping=True), str(percent)+"%"              
   

def getCategory(name):
    """ extracts category from a metric name""" 
    category =  name.split('.',2)[1] # strip the first field category
    #limit categories until we get a more consistent naming convention in place
    categories = {
        'as':"ActionScript",
        'rend':"Rendering",
        'network':"Network",
        'mem':"Memory",
        'tlm':"Telemetry"
    }
    category = categories.get(category,"Player")
    return category   

def deltafunction(a,b):
    if a and b:
        return b-a

class IndexList(list):
    """
    This class indexes a second list into frames seperated by a marker tag
    the interval between each frame can then be evaluated statistically
    an interval is defined as the being between two markers
    There is special 0 interval before the first marker and a final interval after the last marker
    By passing all metrics to addFrame, this can track the pre and post intervals
    Index 0 in this list refers to the interval before the first frame
    So the indexes are sort-of 1 based
    """
    def __init__(self,marker):
        self.marker = marker
        self.positions = []
        #self.deltas = []  
        self.startTime = 0
        self.endTime = 0
        self.foundMarker = False
        #self.min = 0
        #self.max = 0 
        # set false when data is pre-processed 
        # (delta values already resolved


    def __getslice__(self, i, j):
        newIndex = IndexList(self.marker)
        newIndex.positions = self.positions[i:j]
        newIndex.extend(list.__getslice__(self,i,j))
        if len(newIndex):
            newIndex.startTime = newIndex[0]
            newIndex.endTime = newIndex[-1]
        return newIndex
    
    def getIndexByTime(self, time):
        """ finds the frame interval that contains the time given """
        return bisect.bisect(self,time)
          
    def getPositionByTime(self, time):
        """ finds the frame interval that contains the time given """
        index = self.getIndexByTime(time)
        return self.positions[index]
                
    def getPositionByIndex(self, index):
        """ indexes by frame number, 0 is before the first, >= len is after the last """
        if index < 0 or index >= len(self):
            return -1  # just reference the last frame
        elif index == 0:
            return 0
        return self.positions[index-1]

    def getInterval(self, index):
        if index==0:
            return self[0]-self.startTime
        if index >= len(self):
            return self.endTime-self[-1]
        return self[index]-self[index-1]
         
    def addFrame(self, name, pos, time):
        if name == self.marker:
            self.append(time)
            #print "adding marker",name,time
            self.positions.append(pos)
        if self.startTime==0:
            self.startTime = time
        self.endTime = time
            
    def addOldFrame(self, name, pos, time):  
        # this is a little funky
        # we measure frames from one .enter to another
        # this could be made more accurate, but I'm trying replicate what is done in flashMonitor here.
        # A 'frame' includes all the metrics from the current .enter until the next .enter
        # the reason for this is to ensure that we account correctly for nested metrics
        if self.startTime == 0:
            self.startTime = time
        if name == self.marker:
            self.foundMarker = True  # remember that we found the marker, but don't add until .enter
        if name == '.enter':
            if self.foundMarker:    
                self.append(self.startTime)
                self.positions.append(pos)
                self.foundMarker = False
                self.startTime = time
     
    """      
    def getResults(self,dataList):
 
        self.deltas =  map(deltafunction, self[:-1],self[1:])
        self.median = self.getMedian();
        self.average, self.stdev = self.meanstdv();
        if len(self.deltas) > 0:
            self.min = min(self.deltas)
            self.max = max(self.deltas)  
        print len(self), self.min, self.max, self.average, self.median, self.stdev
        for i in range(len(self.deltas)-1):
            d = self.deltas[i]
            if d < self.average-self.stdev or d > self.average+self.stdev:
                print "\nLong Interval at frame:", i, timeStr(self[i]), ":", timeStr(self.deltas[i])
                self.report(dataList, i)

    def getMedian(self):
        if len(self.deltas) < 1:
            return 0
        values = sorted(self.deltas)
        print min(values), max(values)
        if len(values) % 2 == 1:
            return values[(len(values)+1)/2-1]
        else:
            lower = values[len(values)/2-1]
            upper = values[len(values)/2]
            return (float(lower + upper)) / 2         
    """
            
    def meanstdv(self):
        from math import sqrt
        n, mean, std = len(self), 0.0, 0.0
        if n >0:
            for i in range(len(self)):
                mean = mean + self.getInterval(i)
            mean = mean / float(n)
            for i in range(1,len(self)):
                std = std + (self.getInterval(i) - mean)**2
            std = sqrt(std / float(n))
        return mean, std  
   
class swfInstance():
    def __init__(self):
        self.reset()
        
    def reset(self):
        self.name = ""
        self.rate = 0
        self.startTime = 0
        self.telemetryVersion = 0
        self.date = gmtime();
        self.infoCount = 0
        #self.dataList = []
        self.timeLine = []
        # index by enterframe and render events so we can report both FPS rates
        self.indexList = IndexList(options.frameMarker)
        self.renderList = IndexList(".rend.screen")
        self.time = 0
        self.frameMarker = kSwfFrameMarker
        self.totalSpan = 0  # total span time recorded (before subtracting nested time)
        self.inactiveTest = None
        self.activeTest = None
        self.lastSpanTime = 0
        self.streaming = True
        self.metricCount = 0
        self.profstack = []
        self.capabilities = {}
        
    def haveInfo(self):  # got what we need already    
        return self.infoCount > 4; 

    def swfInfo(self):  # got what we need already 
        return {
            'name':self.name,
            'rate':self.rate,
            'start':self.startTime,
            'telemetry_version':self.telemetryVersion }
            

    def getInfoStr(self):  # got what we need already 
        #return str(self.swfInfo())
        s =  "Swf Name = " + self.name           
        if self.rate:
            s += "\nSWF Rate = " + str(round(1000000/self.rate)) + " fps"
        s += "\nTelemetry version = " + str(self.telemetryVersion)   
        #s += "\n" 
        return s 

    def printMetric(self, metric):
        depth = metric.get('depth',0)
        name = metric['name']
        out = timeStr(metric.get('time',0))+": "
        out += "  " * depth
        out += name
        if "span" in metric:
            out += " = " + str(metric['span'])
        if "value" in metric:
            if type(metric['value']) == str:
                out += " = " + '"'+metric['value']+'"'
            else:
                out += " = " + str(metric['value'])
        print(out)

             
    def addMetric(self,metric):   
        #print "addMetric", metric
        self.metricCount += 1
        """ Adds a metric to the swf metric collection """
        # keep running telemetry time
        # support new metrics that reference time as "delta"
        if "delta" in metric:
            # for stream content we adjust delta values
            # flasmMonitor saved content is already adjusted
            if self.streaming:
                self.lastSpanTime += metric["delta"]
                metric["time"] = self.lastSpanTime
            else:
                metric["time"] = metric['delta']
            
        if "time" in metric:
            self.time = metric["time"] 
            #if self.time < 100:
            #    print "INVALID TIME", metric
        else:
            metric["time"] = self.time
            
        name = metric['name']
        
        if name.startswith(".prof."):
           if name==".prof.enter.time":
                m = {'name':"none",'time':self.time,'span':0}
                #print "profstack push", m
                self.profstack.append(m)
                return
           elif name==".prof.enter.name":
                self.profstack[-1]["name"] = ".as."+metric['value']
                return
           elif name==".prof.exit.time":
                if len(self.profstack) < 1: 
                    print("profstack empty error", metric)
                    return
                metric = self.profstack.pop()
                metric["span"] = self.time - metric["time"]
                metric["time"] = self.time
                if metric["span"] < 0:
                    print(("profstack invalid pop ",metric))
             
        #self.printMetric(metric);
        
        self.flatten(metric,self.timeLine)

        #self.dataList.append(metric)
        
        if self.haveInfo():
            return; 
            
        if name.startswith(".swf."):
            if name==".swf.name":
                self.name = metric['value']
                self.infoCount += 1
            elif name==".swf.rate":
                self.rate = metric['value']
                self.infoCount += 1
            elif name==".swf.start":
                self.startTime = metric['time']
                self.time = self.startTime
                self.infoCount += 1
        elif name.startswith(".tlm."):
            if name == ".tlm.version":
                self.telemetryVersion = metric['value']
                self.infoCount += 1
            elif name==".tlm.date":
                self.date = datetime.fromtimestamp(metric['value']/1000);
                self.infoCount += 1
            elif name==".tlm.inactive":
                self.inactiveTest = metric['span']
            elif name==".tlm.active":
                self.activeTest = metric['span'] 
        elif name==".capabilities":
            from urllib.parse import urlparse
            from urllib.parse import unquote
            url = urlparse("http://foo.bar?"+unquote(metric['value']))
            print("cap", url)
            self.capabilities = dict([part.split('=') for part in url[4].split('&')]) 
            #self.capabilities = dict([part.split('=') for part in url.split('&')]) 
            print("capabilities",self.capabilities) 

    def flatten(self, metric, timeLine):
        """ creates a flat list by inserting fragments for nested logic
            all times are normalized to start time+span
            this provides accurate measurment of metric and category time wihtin any arbitray time span
        """
        #print "flattening", metric
        name = metric['name']
        if "span" in metric:
            span = metric['span']
            end =  metric['time']
            if span < 0:
                print("Invalid Metric span")
            #print "flatten metric", metric
            start = end-span
            self.indexList.addFrame(name, len(timeLine), start)
            self.renderList.addFrame(name, len(timeLine), start)
            self.totalSpan += span  # track this for sanity check
            childSpanSum = 0
            # find all children of this span 
            childIndex = -1
            i = len(timeLine) - 1
            while i >=0:
                if 'time' in timeLine[i]:
                    if timeLine[i]['time'] >= start:
                        childIndex = i
                    else:
                        break
                i -= 1
                
            if childIndex > -1:
                # remove children from the list
                children = timeLine[childIndex:]
                timeLine[childIndex:] = []
                
                # add children back in, inserting fragmented spans as needed
                for child in children:
                    if 'span' in child:
                        childStart = child['time']
                        childSpan = child['span']
                        childSpanSum += childSpan
                        if childStart > start:  # insert fractional metric for extra space
                            newChildSpan = childStart-start
                            newChild = {'time':start,'span':newChildSpan,'name':name,'depth':0}
                            #if not primarySpan: newChild['secondary'] = True  #mark secondary (fragment) references
                            #print "newChild added", newChild
                            timeLine.append(newChild)
                            span -= newChildSpan
                        #print "child Added",child
                        child['depth'] += 1
                        timeLine.append(child)
                        span -= childSpan
                        start = childStart+childSpan
                    else:
                        child['depth'] += 1
                        timeLine.append(child)  
                        
            if childSpanSum > metric['span']:
                print("Invalid Child span", metric['span'], childSpanSum)          
            timeLine.append({'time':start,'span':span,'name':name,'depth':0})             
            #print "appended", {'time':start,'span':span,'name':name}
        else:
            # add non-span metrics
            self.indexList.addFrame(name, len(timeLine), self.time)
            self.renderList.addFrame(name, len(timeLine), self.time)
            m = dict(metric)
            m['depth'] = 0
            timeLine.append(m)

    def validate(self):
        for i in range(len(self.timeLine)):
            if not i: continue
            m = self.timeLine[i-1]
            m2 = self.timeLine[i]
            t = m['time']
            t2 = m2['time']
            if t2 < t: print("time is less @ ", i, m, m2)
            if 'span' in m:
                s = m['span']
                if t+s > t2: print("span is too long @ ", i, m, m2)

    
    def process(self):
        print("Date = " + str(self.date))   
        print(self.getInfoStr());
        print("Startup Time = ", timeStr(self.startTime))
        
        if options.range:
            try:
                rstart, rend = options.range.split(":")
                rstart = int(rstart)
                rend = int(rend)
            except:
                print("Invalid range %s, must be in start:end format" % options.range)
                return            
            pos = self.indexList.getPositionByIndex(rstart)
            pos2 = self.indexList.getPositionByIndex(rend)
            indexList = self.indexList[rstart:rend]
            selection = self.timeLine[pos:pos2]
            if not len(selection): 
                print("No metrics in Range %d:%d" % (rstart, rend))
                return
            t1 = selection[0].get('time',0)
            t2 = selection[-1].get('time',0)
            renderPos1 = self.renderList.getIndexByTime(t1)
            renderPos2 = self.renderList.getIndexByTime(t2)
            print("Range %d:%d (%s-%s)" % (rstart, len(indexList), timeStr(t1), timeStr(t2)))
            renderList = self.renderList[renderPos1:renderPos2]
        else:
            rstart = 0
            rend = len(self.indexList)
            selection = self.timeLine  
            renderList = self.renderList 
            indexList = self.indexList      
        
        print("Metric Count = %d" % len(selection))
        print("Frame Count = %d" % len(indexList))
        print("Render Count = %d" % len(renderList))
   
        #self.validate()
        reporter = Reporter(selection);
        
        runTime = reporter.endTime-reporter.startTime
        print("Run Time = ", timeStr(runTime))
        print("Time in Player = ", timeStr(reporter.getSpan()))
        if runTime:
            print("Load = %.2f%%" % ((reporter.getSpan()/runTime)*100))
        average, stdev = indexList.meanstdv();
        if average:
            #print average, len(renderList)
            print("Frame FPS = %.2f" % (1000000/average))
        average, stdev = renderList.meanstdv();
        if average:
            #print average, len(renderList)
            print("Render RPS = %.2f" % (1000000/average))
        if self.inactiveTest:
            print("Telemetry Inactive Test", self.inactiveTest)
        if self.activeTest:
            print("Telemetry Active Test", self.activeTest)
            
        
        reporter.report()
                 
        if options.showFrames: 
            for index in range(rstart,rend):
                self.rangeReport(index, index+1)
        print()                

    def rangeReport(self, index1, index2):
        pos = self.indexList.getPositionByIndex(index1)
        pos2 = self.indexList.getPositionByIndex(index2)
        #print "RANGE", index1, index2, pos, pos2
        frame = self.timeLine[pos:pos2]
        reporter = Reporter(frame)
        load = 0
        if reporter.getSpan() and reporter.getInterval():
            load = ((reporter.getSpan()/reporter.getInterval())*100)
        if options.loadFilter and load < options.loadFilter: return
        if index1+1 == index2:
            print("\nReport for frame #", index1)  
        else:  
            print("\nReport for range %d-%d" % index1, index2)   
        frameTime =frame[0].get('time',0)
        print("Time: %s (%d/%d)"% (timeStr(frameTime),reporter.getInterval(),reporter.getSpan()))
        print("Load %.2f%%" % load)

        reporter.report()
        if options.showMetrics:
            for m in frame:
                self.printMetric(m)                

if __name__ == '__main__':
    import sys

    parser = OptionParser()
    parser.add_option("-f", "--frames", 
        action="store_true", dest="showFrames", default=False,
        help ="generate report for all frames")
    parser.add_option("-d", "--dump",
        action="store_true", dest="hexDump", default=False,
        help="generate amf3 hex dump while parsing")
    parser.add_option("-s", "--summary",
        action="store_true", dest="metrics", default=False,
        help="show metrics summary")
    parser.add_option("-a", "--all",
        action="store_true", dest="showMetrics", default=False,
        help="show all metrics")
    parser.add_option("-m", "--memory",
        action="store_true", dest="showMemory", default=False,
        help="show Memory Stats")
    parser.add_option("-l", "--load",
        action="store",type="int", dest="loadFilter", default=0,
        help="filter by load level")
    parser.add_option("", "--range",
        action="store", dest="range", default="",
        help="set range of frames RANGE = start:end")

    (options, args) = parser.parse_args()
    options.frameMarker =  ".swf.frame"  # change this to redefine a frame
    
    for filename in args:
        print(("\nReport for: "+filename ))
        
        file = open(filename, 'rb')
        data = file.read()
        file.close()
      
        tlm = amf3reader.amf3reader()
        tlm.verbose = options.hexDump
        
        tlm.setData(data)
        
        swf = swfInstance()
        metric = tlm.readMetric() 
        
        #print type(metric) 
        #print metric      
        if type(metric) == list: # we read all metrics as one list
            swf.streaming = False
            for m in metric:
                swf.addMetric(m)
        else:        
            swf.streaming = True
            while metric:
                swf.addMetric(metric)
                metric = tlm.readMetric()
        swf.process()
        
