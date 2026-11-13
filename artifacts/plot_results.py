import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import csv
from pathlib import Path
out=Path(__file__).parent
rows=[('Baseline','667ce8d4',896396908,204,True),('E040','57a62d9e',893766672,641,False),('E046','49eb0909',893028824,1854,False),('Official87','87f0aaf3',890917150,216,True),('E052','6ac627e5',890897406,214,False),('E056','e50c8152',890907278,210,False)]
with (out/'research_results.csv').open('w') as f:
 w=csv.writer(f);w.writerow(['version','commit','score','union_failures_per_100000','official_local_pass']);w.writerows(rows)
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':11})
fig,(ax,az)=plt.subplots(1,2,figsize=(15,6.8),gridspec_kw={'width_ratios':[1.15,1]})
fig.suptitle('secp256k1 point-add circuit: cost vs. error',fontsize=20,fontweight='bold',y=.98)
colors=['#52677d','#8597ab','#8597ab','#16836c','#d88717','#7955ad']
for i,(name,commit,score,fail,valid) in enumerate(rows):
 rate=fail/1000
 ax.scatter(score/1e6,rate,s=150 if valid else 100,marker='*' if valid else 'o',color=colors[i],zorder=4)
 offsets=[(-12,20),(12,10),(12,-8),(10,42),(10,-39),(10,-4)]
 text=f'{name} | {commit}\n{score:,} | {rate:.3f}%'
 ax.annotate(text,(score/1e6,rate),xytext=offsets[i],textcoords='offset points',ha='right' if i==0 else 'left',fontsize=9,arrowprops={'arrowstyle':'-','color':colors[i]},color=colors[i])
ax.set_xlim(890.2,897.1);ax.set_ylim(0,2.13);ax.set_title('Overview: lower and left is better',pad=15)
for i in [3,4,5]:
 name,commit,score,fail,valid=rows[i];p=fail/100000;z=1.96;n=100000
 center=(p+z*z/(2*n))/(1+z*z/n);half=z*((p*(1-p)/n+z*z/(4*n*n))**.5)/(1+z*z/n)
 az.errorbar(score/1e6,p*100,yerr=[[100*(p-center+half)],[100*(center+half-p)]],fmt='*' if valid else 'o',markersize=15 if valid else 9,color=colors[i],capsize=5,zorder=3)
 offsets={3:(-12,45),4:(10,-42),5:(12,-44)}
 roles={3:'Best locally official-valid',4:'Lowest research score',5:'Error-efficient candidate'}
 az.annotate(f'{name} | {commit}\n{score:,} | {fail/1000:.3f}%\n{roles[i]}',(score/1e6,p*100),xytext=offsets[i],textcoords='offset points',ha='right' if i==3 else 'left',fontsize=9,color=colors[i],arrowprops={'arrowstyle':'-','color':colors[i]})
az.set_xlim(890.888,890.934);az.set_ylim(.145,.285);az.set_title('Current leaders (zoom; 95% Wilson intervals)',pad=15)
for a in [ax,az]:
 a.set_xlabel('Score = peak qubits × rounded mean Toffoli (millions)');a.set_ylabel('Union error rate on fixed 100K development inputs (%)');a.grid(alpha=.2);a.spines[['top','right']].set_visible(False);a.ticklabel_format(useOffset=False,axis='x')
fig.text(.07,.045,'Stars: passed local official 9,024-shot evaluation. Circles: failed that evaluation.\nError = value OR phase OR ancilla failure on common independent inputs; official pass does not imply zero population error.\nE052/E056 error-rate improvements over Official87 are not statistically resolved. Source: research checkpoints, 2026-11-06.',fontsize=10,color='#485463',linespacing=1.5)
fig.subplots_adjust(left=.07,right=.97,bottom=.23,top=.86,wspace=.29)
fig.savefig(out/'research_results.png',dpi=180)
fig.savefig(out/'research_results.svg')
