import { Component, signal, OnInit } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { BioISService, Indicator, IndicatorResponse } from './biois.service';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, CommonModule, FormsModule],
  templateUrl: './app.html',
  styleUrl: './app.css'
})
export class App implements OnInit {
  protected readonly title = signal('BioIS - Biodiversity Indicator Service');
  indicators = signal<Indicator[]>([]);
  selectedIndicator = signal<string>('');
  bbox = signal<string>('-180,-90,180,90');
  result = signal<IndicatorResponse | null>(null);
  loading = signal<boolean>(false);
  error = signal<string | null>(null);

  constructor(private bioISService: BioISService) {}

  async ngOnInit() {
    try {
      const indicators = await this.bioISService.getIndicators();
      this.indicators.set(indicators);
      if (indicators.length > 0) {
        this.selectedIndicator.set(indicators[0].name);
      }
    } catch (err) {
      this.error.set('Failed to load indicators');
      console.error(err);
    }
  }

  async calculateIndicator() {
    if (!this.selectedIndicator()) return;
    
    this.loading.set(true);
    this.error.set(null);
    try {
      const result = await this.bioISService.calculateIndicator({
        indicator_type: this.selectedIndicator(),
        bbox: this.bbox()
      });
      this.result.set(result);
    } catch (err) {
      this.error.set('Failed to calculate indicator');
      console.error(err);
    } finally {
      this.loading.set(false);
    }
  }
}
