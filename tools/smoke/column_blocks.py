#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Prints the block runs of one column of a generated world, top down (diagnostic, P1-13).

    column_blocks.py <world folder> <x> <z> <y_top> <y_bottom>

Reads the region file directly (no server): each line is the y where the block changes and the
block now. Shows what the generator really built where `/deeptime probe column`'s surfaceY and the
world data's surface disagree."""
import sys, struct, zlib, io
# minimal NBT reader
def rd(f):
    def u(fmt):
        n = struct.calcsize(fmt); return struct.unpack(">"+fmt, f.read(n))[0]
    def string(): return f.read(u("H")).decode("utf8","replace")
    def payload(t):
        if t==1: return u("b")
        if t==2: return u("h")
        if t==3: return u("i")
        if t==4: return u("q")
        if t==5: return u("f")
        if t==6: return u("d")
        if t==7: n=u("i"); return f.read(n)
        if t==8: return string()
        if t==9:
            et=u("B"); n=u("i"); return [payload(et) for _ in range(n)]
        if t==10:
            d={}
            while True:
                tt=u("B")
                if tt==0: return d
                k=string(); d[k]=payload(tt)
        if t==11: n=u("i"); return list(struct.unpack(">%di"%n, f.read(4*n)))
        if t==12: n=u("i"); return list(struct.unpack(">%dq"%n, f.read(8*n)))
    t=u("B"); string(); return payload(t)
def chunk(region, cx, cz):
    d=open(region,"rb").read()
    i=(cx&31)+(cz&31)*32
    off=struct.unpack(">I", d[4*i:4*i+4])[0]
    if off==0: return None
    o=(off>>8)*4096; ln=struct.unpack(">I", d[o:o+4])[0]; c=d[o+4]
    return rd(io.BytesIO(zlib.decompress(d[o+5:o+4+ln])))
X,Z=int(sys.argv[2]),int(sys.argv[3])
rx,rz=X>>9,Z>>9
ch=chunk(f"{sys.argv[1]}/region/r.{rx}.{rz}.mca", (X>>4)&31 | 0, (Z>>4)&31)
if ch is None: print("no chunk"); sys.exit()
ymax=int(sys.argv[4]); ymin=int(sys.argv[5])
secs={s["Y"]:s for s in ch["sections"]}
lx,lz=X&15,Z&15
out=[]
for y in range(ymax,ymin-1,-1):
    s=secs.get(y>>4)
    if not s or "block_states" not in s: out.append((y,"?")); continue
    bs=s["block_states"]; pal=bs["palette"]
    if len(pal)==1: name=pal[0]["Name"]
    else:
        bits=max(4,(len(pal)-1).bit_length()); per=64//bits
        idx=((y&15)*16+lz)*16+lx
        w=bs["data"][idx//per]&0xFFFFFFFFFFFFFFFF
        name=pal[(w>>((idx%per)*bits))&((1<<bits)-1)]["Name"]
    out.append((y,name))
prev=None
for y,n in out:
    if n!=prev: print(y,n); prev=n
