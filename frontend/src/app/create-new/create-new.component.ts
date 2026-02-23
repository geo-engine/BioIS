import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  form,
  FormField,
  min,
  max,
  required,
  applyEach,
  validate,
  ChildFieldContext,
  ValidationResult,
} from '@angular/forms/signals';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatButtonModule } from '@angular/material/button';
import {
  NDVIProcessInputs,
  PointGeoJsonInputMediaType,
  PointGeoJsonType,
  ProcessesApi,
  Response,
} from '@geoengine/biois';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { UserService } from '../user.service';
import { Router } from '@angular/router';

@Component({
  selector: 'app-create-new',
  imports: [
    CommonModule,
    FormField,
    MatButtonModule,
    MatCheckboxModule,
    MatFormFieldModule,
    MatInputModule,
    MatSelectModule,
  ],
  templateUrl: './create-new.component.html',
  styleUrls: ['./create-new.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CreateNewComponent {
  readonly userService = inject(UserService);
  readonly router = inject(Router);

  readonly formModel = signal({
    inputs: {
      coordinate: {
        value: {
          type: PointGeoJsonType.Point,
          coordinates: [0, 0],
        },
        mediaType: PointGeoJsonInputMediaType.ApplicationGeojson,
      },
      year: 2014,
      month: 1,
    } as NDVIProcessInputs,
    outputs: {
      ndvi: true,
      kNdvi: true,
    },
  });
  readonly form = form(this.formModel, (schema) => {
    applyEach(schema, (field) => required(field));
    applyEach(schema.inputs, (field) => required(field));

    validate(schema.inputs.coordinate.value.coordinates, coordinateValidator);

    min(schema.inputs.year, 2014);
    max(schema.inputs.year, 2014);

    min(schema.inputs.month, 1);
    max(schema.inputs.month, 6);
  });

  async onSubmit(): Promise<void> {
    const processApi = new ProcessesApi(this.userService.apiConfiguration());

    await processApi.executeNdvi({
      inputs: this.formModel().inputs,
      outputs: outputForRequest(this.formModel().outputs),
      response: Response.Document,
    });

    await this.router.navigate(['/app/results']);
  }
}

function coordinateValidator(control: ChildFieldContext<Array<number>>): ValidationResult {
  const coordinates = control.value();
  if (coordinates?.length !== 2) {
    return { kind: 'invalidCoordinate', message: 'Coordinates must be an array of two numbers.' };
  }

  const [longitude, latitude] = coordinates;

  if (longitude < -180 || longitude > 180) {
    return { kind: 'invalidLongitude', message: 'Longitude must be between -180 and 180.' };
  }
  if (latitude < -90 || latitude > 90) {
    return { kind: 'invalidLatitude', message: 'Latitude must be between -90 and 90.' };
  }

  return null; // Valid
}

function outputForRequest(output: { ndvi: boolean; kNdvi: boolean }): {
  ndvi?: Record<never, never>;
  kNdvi?: Record<never, never>;
} {
  return {
    ndvi: output.ndvi ? {} : undefined,
    kNdvi: output.kNdvi ? {} : undefined,
  };
}
