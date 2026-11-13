from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from matplotlib.ticker import FuncFormatter
out=Path(__file__).parent
labels=['Official87\n87f0aaf3','E052 (Ours)\n6ac627e5']
scores=[890917150,890897406]
rates=[.216,.214]
colors=['#16836c','#d88717']
plt.rcParams.update({'font.family':'DejaVu Sans','font.size':12})
fig,axes=plt.subplots(1,2,figsize=(11,6))
fig.suptitle('Official87 vs E052',fontsize=21,fontweight='bold',y=.96)
for ax,vals,title,ylim,fmt in [(axes[0],scores,'Score (lower is better)',1.04e9,lambda x:f'{x:,}'),(axes[1],rates,'Error rate (lower is better)',.255,lambda x:f'{x:.3f}%')]:
 bars=ax.bar(labels,vals,color=colors,width=.53)
 ax.set_title(title,pad=17)
 ax.set_ylim(0,ylim)
 ax.bar_label(bars,labels=[fmt(v) for v in vals],padding=8,fontsize=12,fontweight='bold')
 ax.spines[['top','right']].set_visible(False)
 ax.yaxis.grid(True,alpha=.18)
 ax.set_axisbelow(True)
axes[0].yaxis.set_major_formatter(FuncFormatter(lambda v,p:f'{v/1e6:,.0f}M'))
axes[0].set_ylabel('Peak qubits × rounded mean Toffoli')
axes[1].yaxis.set_major_formatter(FuncFormatter(lambda v,p:f'{v:.2f}%'))
axes[1].set_ylabel('Union failures / 100,000 fixed development inputs')
fig.text(.08,.09,'Official87: passed local official evaluation. E052: failed local official evaluation.\nError rates use the same independent 100K development corpus (216 vs 214 failures).\nE052 score: 19,744 lower (0.00222%). Error-rate difference is not statistically resolved.',fontsize=10,color='#485463',linespacing=1.5)
fig.subplots_adjust(left=.09,right=.97,top=.82,bottom=.26,wspace=.35)
fig.savefig(out/'e052_vs_official_bars.png',dpi=200)
fig.savefig(out/'e052_vs_official_bars.svg')
