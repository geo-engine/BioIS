import { Injectable, Signal, signal } from '@angular/core';
import { TITLE } from '../app.routes';

@Injectable({
  providedIn: 'root',
})
export class TitleService {
  private readonly _title = signal(TITLE);

  get title(): Signal<string> {
    return this._title;
  }

  set title(newTitle: string) {
    this._title.set(newTitle);
  }
}
