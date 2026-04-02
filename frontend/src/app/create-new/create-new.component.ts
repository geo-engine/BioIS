import { ChangeDetectionStrategy, Component, inject, resource, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import {
  form,
  FormField,
  min,
  max,
  required,
  applyEach,
  validateTree,
  FieldTree,
  validate,
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
import { MatIcon } from '@angular/material/icon';
import { MatTooltipModule } from '@angular/material/tooltip';

@Component({
  selector: 'app-create-new',
  imports: [
    CommonModule,
    FormField,
    MatButtonModule,
    MatCheckboxModule,
    MatFormFieldModule,
    MatIcon,
    MatInputModule,
    MatSelectModule,
    MatTooltipModule,
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
      year: 2020,
      month: 1,
    } as NDVIProcessInputs,
    outputs: {
      ndvi: true,
      kNdvi: true,
    },
  });
  readonly form = form(this.formModel, (schema) => {
    applyEach(schema, (field) => required(field, { message: 'This field is required.' }));
    applyEach(schema.inputs, (field) => required(field, { message: 'This field is required.' }));

    applyEach(schema.inputs.coordinate.value.coordinates, (field) => {
      required(field, { message: 'This field is required.' });
    });

    validateTree(schema.inputs.coordinate.value.coordinates, (fields) => {
      const coordinates = fields.value();
      if (coordinates?.length !== 2) {
        return {
          kind: 'invalidCoordinate',
          message: 'Coordinates must be an array of two numbers.',
        };
      }

      const [longitude, latitude] = coordinates;

      const arrayTree: FieldTree<number[], string | number> = fields.fieldTree;

      const [longitudeField, latitudeField] = [arrayTree[0], arrayTree[1]];

      if (longitude < -180 || longitude > 180) {
        return {
          kind: 'invalidLongitude',
          message: 'Longitude must be between -180 and 180.',
          fieldTree: longitudeField,
        };
      }
      if (latitude < -90 || latitude > 90) {
        return {
          kind: 'invalidLatitude',
          message: 'Latitude must be between -90 and 90.',
          fieldTree: latitudeField,
        };
      }

      return;
    });

    min(schema.inputs.year, 2020, { message: 'Year must be 2014 or later.' });
    max(schema.inputs.year, 2020, { message: 'Year must be 2014 or earlier.' });

    min(schema.inputs.month, 1, { message: 'Month must be 1 or later.' });
    max(schema.inputs.month, 12, { message: 'Month must be 6 or earlier.' });

    validate(schema.outputs, (outputs) => {
      const { ndvi, kNdvi } = outputs.value();
      if (!ndvi && !kNdvi) {
        return {
          kind: 'noOutputSelected',
          message: 'At least one output must be selected.',
        };
      }
      return;
    });
  });

  readonly description = resource({
    loader: () => {
      const processApi = new ProcessesApi(this.userService.apiConfiguration());
      return processApi.process('ndvi');
    },
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

function outputForRequest(output: { ndvi: boolean; kNdvi: boolean }): {
  ndvi?: Record<never, never>;
  kNdvi?: Record<never, never>;
} {
  return {
    ndvi: output.ndvi ? {} : undefined,
    kNdvi: output.kNdvi ? {} : undefined,
  };
}
