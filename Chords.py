#!/usr/bin/python
# -*- coding: utf-8 -*-

import sys,time
from PyQt5 import QtGui, QtCore

class ScriptClass(QtGui.QWidget):
  def __init__(self, parent=None):
    QtGui.QWidget.__init__(self, parent)

    self.setWindowTitle('Chords : Transforms note into chord')

    nh = 6
    ng = nh - 1

    self.chord = [
      ('Major', [0, 7, 12, 16, 19, 24], [False, False, False, False, False, False]),
      ('Minor', [0, 7, 12, 15, 19, 24], [False, False, False, False, False, False]),
      ('Octave', [0, 12, 24, 36, 48, 60], [False, False, True, True, True, True]),
      ('dim', [0, 6, 12, 15, 19, 24], [False, False, False, False, True, True]),
      ('aug', [0, 8, 12, 16, 20, 24], [False, False, False, False, False, False]),
      ('sus', [0, 7, 12, 17, 19, 24], [False, False, False, False, False, False]),
      ('6', [0, 7, 12, 16, 21, 24], [False, False, False, False, False, False]),
      ('7', [0, 7, 10, 16, 19, 24], [False, False, False, False, False, False]),
      ('maj7', [0, 7, 11, 16, 19, 24], [False, False, False, False, False, True]),
      ('9', [0, 7, 10, 16, 19, 26], [False, False, False, False, False, False]),
      ('add9', [0, 7, 14, 16, 19, 24], [False, False, False, False, False, False]),
      ('m6', [0, 7, 12, 15, 21, 24], [False, False, False, False, False, False]),
      ('m7', [0, 7, 12, 15, 22, 24], [False, False, False, False, False, False]),
      ('mmaj7', [0, 7, 11, 15, 19, 24], [False, False, False, False, False, True]),
      ('m9', [0, 7, 10, 15, 19, 26], [False, False, False, False, False, False]),
      ('11', [0, 7, 10, 17, 19, 26], [False, False, False, False, False, False]),
      ('7sus4', [0, 7, 10, 17, 19, 24], [False, False, False, False, False, False]),
      ('13', [0, 4, 10, 14, 21, 24], [False, False, False, False, False, True]),
      ('6add9', [0, 4, 9, 14, 19, 24], [False, False, False, False, False, False]),
      ('-5', [0, 7, 12, 18, 24, 28], [False, True, False, False, False, False]),
      ('7-5', [0, 12, 18, 22, 24, 28], [False, False, False, False, False, False]),
      ('7maj5', [0, 12, 20, 22, 28, 32], [False, False, False, False, False, False]),
      ('maj9', [0, 7, 11, 16, 19, 26], [False, False, False, False, False, False])
      ]

    self.chordEdit = QtGui.QComboBox()
    for nm, rp, mt in self.chord: self.chordEdit.addItem(nm)
#    self.chordEdit.addItems([, '', '', 'sus', '6','9', 'add9', 'm6', 'm7', 'mmaj7', 'm9', '11', '7sus4'])

    self.rp = [QtGui.QSpinBox() for i in range(nh)]
    for o in self.rp: o.setRange(-127, 127)

    arp = QtGui.QSpinBox()
    arp.setRange(-127, 127)
    self.connect(arp,  QtCore.SIGNAL('valueChanged(int)'), self.setRP) 

    self.tg = [QtGui.QSpinBox() for i in range(ng)]
    for o in self.tg: o.setRange(-999999, 999999)
    for o in self.tg: o.setValue(8)

    atg = QtGui.QSpinBox()
    atg.setRange(-999999, 999999)
    self.connect(atg,  QtCore.SIGNAL('valueChanged(int)'), self.setTG)

    self.tglr = [QtGui.QDoubleSpinBox() for i in range(ng)]
    for o in self.tglr: o.setValue(0)

    atglr = QtGui.QDoubleSpinBox()
    self.connect(atglr,  QtCore.SIGNAL('valueChanged(double)'), self.setTGLR) 

    self.mt = [QtGui.QCheckBox() for i in range(nh)]
    amt = QtGui.QCheckBox()
    self.connect(amt,  QtCore.SIGNAL('clicked(bool)'), self.setMT)

    self.vg = [QtGui.QSpinBox() for i in range(ng)]
    for o in self.vg: o.setRange(-127, 127)
    for o in self.vg: o.setValue(-2)

    avg = QtGui.QSpinBox()
    avg.setRange(-127, 127)
    self.connect(avg,  QtCore.SIGNAL('valueChanged(int)'), self.setVG)

    self.lr = [QtGui.QDoubleSpinBox() for i in range(nh)]
    for o in self.lr: o.setValue(1)

    self.alr = QtGui.QDoubleSpinBox()
    self.connect(self.alr,  QtCore.SIGNAL('valueChanged(double)'), self.setLR) 


    grid = QtGui.QGridLayout()
    grid.setSpacing(3)

    allCol = 2 * nh
    allCol2 = allCol + 1
    optCol = allCol2 + 1

#     grid.addWidget(QtGui.QLabel('Transforms note into chord:'), 1, 0)
    grid.addWidget(self.chordEdit, 1, 0)
    for i in range(nh):
      grid.addWidget(QtGui.QLabel('H'+str(i + 1)), 1, 2 * i + 1)

    grid.addWidget(QtGui.QLabel('All'), 1, allCol2)

    grid.addWidget(QtGui.QLabel('Relavive Pitch'), 2, 0)
    i = 1
    for o in self.rp:
      grid.addWidget(o, 2, i)
      i += 2

    grid.addWidget(arp, 2, allCol2)

    grid.addWidget(QtGui.QLabel('Tick Gap'), 3, 0)
    i = 2
    for o in self.tg:
      grid.addWidget(o, 3, i)
      i += 2

    grid.addWidget(atg, 3, allCol2)

    grid.addWidget(QtGui.QLabel('Tick Gap Length Rate'), 4, 0)
    i = 2
    for o in self.tglr:
      grid.addWidget(o, 4, i)
      i += 2

    grid.addWidget(atglr, 4, allCol2)

    grid.addWidget(QtGui.QLabel('Mute, Velocity Gap'), 5, 0)
    i = 1
    for o in self.mt:
      grid.addWidget(o, 5, i)
      i += 2
    i = 2
    for o in self.vg:
      grid.addWidget(o, 5, i)
      i += 2

    grid.addWidget(amt, 5, allCol)
    grid.addWidget(avg, 5, allCol2)

#    grid.addWidget(QtGui.QLabel('Velocity Gap'), 5, 0)

    grid.addWidget(QtGui.QLabel('Length Factor'), 6, 0)
    i = 1
    for o in self.lr:
      grid.addWidget(o, 6, i)
      i += 2

    grid.addWidget(self.alr, 6, allCol2)

    self.cn = QtGui.QCheckBox('complete')
    self.connect(self.cn,  QtCore.SIGNAL('clicked(bool)'), self.disableLR)
    grid.addWidget(self.cn, 6, optCol)

    self.ln = QtGui.QCheckBox('limited')
    self.connect(self.ln,  QtCore.SIGNAL('clicked(bool)'), self.disableCN)
    grid.addWidget(self.ln, 7, optCol)

    button = QtGui.QPushButton("Execute")

    grid.addWidget(button, 7, 0)

    self.connect(button, QtCore.SIGNAL('clicked()'), self.execute) 
    self.connect(self.chordEdit,  QtCore.SIGNAL('activated(int)'), self.setChord) 
    self.setChord(0)

    self.setLayout(grid)
    self.resize(200, 200)
    button.setFocus()


  def setChord(self, index):
    nm, rp, mt = self.chord[index]

    for i in range(6):
      self.rp[i].setValue(rp[i])
      self.mt[i].setChecked(mt[i])
    
  def setRP(self, val):
    for o in self.rp: o.setValue(val)

  def setTG(self, val):
    for o in self.tg: o.setValue(val)

  def setTGLR(self, val):
    for o in self.tglr: o.setValue(val)

  def setVG(self, val):
    for o in self.vg: o.setValue(val)

  def setMT(self, val):
    for o in self.mt: o.setChecked(val)

  def setLR(self, val):
    for o in self.lr: o.setValue(val)

  def disableLR(self, disable):
    for o in self.lr: o.setDisabled(disable)
    self.alr.setDisabled(disable)
    if disable: self.ln.setChecked(False)

  def disableCN(self, disable):
    if disable:
      self.disableLR(False)
      self.cn.setChecked(False)

  def execute(self):
    testFile = file(sys.argv[1],"r")
    inputEvents = testFile.readlines()
    testFile.close()

    completeNote = self.cn.isChecked()
    limitedNote = self.ln.isChecked()

    begin = 0
    for i in range(6):
      if not self.mt[i].isChecked():
        begin = i
        break

    end = 0
    for i in range(5, -1, -1):
      if not self.mt[i].isChecked():
        end = i
        break

    downNotes = [
     (self.rp[i].value(), self.mt[i].isChecked(), self.tg[i % 5].value(), self.tglr[i % 5].value(), self.lr[i].value(), self.vg[i % 5].value())
     for i in range(begin, end + 1)]

    upNotes = [
     (self.rp[i].value(), self.mt[i].isChecked(), self.tg[max(i - 1, 0)].value(), self.tglr[max(i - 1, 0)].value(), self.lr[i].value(), self.vg[max(i - 1, 0)].value())
     for i in range(end, begin - 1, -1)]

    totalTickGap = 0
    totalTickGapLengthRate = 0

    for i in range(len(downNotes) - 1):
      rp, mt, tg, tglr, lr, vg = downNotes[i]
      totalTickGap += tg
      totalTickGapLengthRate += tglr

    beatLen = 2 * (totalTickGap + totalTickGapLengthRate)
  #get beat length
    for line in inputEvents:
      if line.startswith('BEATLEN'):
        tag,tick = line.split(' ')
        beatLen = int(tick)

    lastTick = -4 * beatLen
    up = False
    
    outputEvents = []
    #loop through events
    for line in inputEvents:
      print "Event : ", line
      if line.startswith('NOTE'):
        tag,tick,pitch,length,velocity = line.split(' ')

        T = int(tick)
        P = int(pitch)
        L = int(length)
        v = int(velocity)
        t = T
        l = L

        dt = T - lastTick

        if dt > beatLen : up = False
        else:
          if dt < beatLen : up = not up

        lastTick = T

        if up :
          notes = upNotes
        else:
          notes = downNotes

        if completeNote and L < beatLen :
          l += totalTickGap + int(totalTickGapLengthRate * L)
          decreaseLen = True
        else:
          decreaseLen = False

        lastTickGap = 0
        for relativePitch, mute, tickGap, tickGapLengthRate, lengthRate, velocityGap in notes :

          if decreaseLen: l -= 2 * lastTickGap
          else: l = int(L * lengthRate)

          if limitedNote: l = min(l, L + T - t)

          if not mute and l > 0 :
            ln = tag+" "+str(t)+" "+str(P + relativePitch)+" "+str(l)+" "+str(v)+"\n"
            outputEvents.append(ln)
            print ln

          lastTickGap = int(tickGap + L * tickGapLengthRate)
          t += lastTickGap
          v += velocityGap

      else:
        outputEvents.append(line)

    testFile = file(sys.argv[1],"w")
    testFile.writelines(outputEvents)
    testFile.close()
        
    quit()


 
app = QtGui.QApplication(sys.argv)
qb = ScriptClass()
qb.show()
sys.exit(app.exec_())

# downNotes = [
#  (self.rp[0].value(), self.mt[0].isChecked(), self.tg[0].value(), self.tglr[0].value(), self.lr[0].value(), self.vg[0].value()),
#  (self.rp[1].value(), self.mt[1].isChecked(), self.tg[1].value(), self.tglr[1].value(), self.lr[1].value(), self.vg[1].value()),
#  (self.rp[2].value(), self.mt[2].isChecked(), self.tg[2].value(), self.tglr[2].value(), self.lr[2].value(), self.vg[2].value()),
#  (self.rp[3].value(), self.mt[3].isChecked(), self.tg[3].value(), self.tglr[3].value(), self.lr[3].value(), self.vg[3].value()),
#  (self.rp[4].value(), self.mt[4].isChecked(), self.tg[4].value(), self.tglr[4].value(), self.lr[4].value(), self.vg[4].value()),
#  (self.rp[5].value(), self.mt[5].isChecked(), 0, 0, self.lr[5].value(), 0)]
#
# upNotes = [
#  (self.rp[5].value(), self.mt[5].isChecked(), self.tg[4].value(), self.tglr[4].value(), self.lr[5].value(), self.vg[4].value()),
#  (self.rp[4].value(), self.mt[4].isChecked(), self.tg[3].value(), self.tglr[3].value(), self.lr[4].value(), self.vg[3].value()),
#  (self.rp[3].value(), self.mt[3].isChecked(), self.tg[2].value(), self.tglr[2].value(), self.lr[3].value(), self.vg[2].value()),
#  (self.rp[2].value(), self.mt[2].isChecked(), self.tg[1].value(), self.tglr[1].value(), self.lr[2].value(), self.vg[1].value()),
#  (self.rp[1].value(), self.mt[1].isChecked(), self.tg[0].value(), self.tglr[0].value(), self.lr[1].value(), self.vg[0].value()),
#  (self.rp[0].value(), self.mt[0].isChecked(), 0, 0, self.lr[0].value(), 0)]
   
#    totalTickGap = self.tg[0].value() + self.tg[1].value() + self.tg[2].value() + self.tg[3].value() + self.tg[4].value()
#    totalTickGapLengthRate = self.tglr[0].value() + self.tglr[1].value() + self.tglr[2].value() + self.tglr[3].value() + self.tglr[4].value()

