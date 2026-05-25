import { Injectable, Signal, signal } from '@angular/core';

@Injectable({
  providedIn: 'root',
})
export class TitleService {
  private readonly _title = signal('BioIS');

  get title(): Signal<string> {
    return this._title;
  }

  set title(newTitle: string) {
    this._title.set(newTitle);
  }
}
