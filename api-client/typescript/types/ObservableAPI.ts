import { ResponseContext, RequestContext, HttpInfo } from "../http/http";
import {
  Configuration,
  ConfigurationOptions,
  mergeConfiguration,
} from "../configuration";
import { Observable, of, from } from "../rxjsStub";
import { mergeMap, map } from "../rxjsStub";
import { AuthCodeResponse } from "../models/AuthCodeResponse";
import { BiodiversitySensitiveAreasProcessOutputs } from "../models/BiodiversitySensitiveAreasProcessOutputs";
import { BiodiversitySensitiveAreasProcessParams } from "../models/BiodiversitySensitiveAreasProcessParams";
import { Conformance } from "../models/Conformance";
import { Execute } from "../models/Execute";
import { GetCreditsResponse } from "../models/GetCreditsResponse";
import { HabitatDistanceProcessOutputs } from "../models/HabitatDistanceProcessOutputs";
import { HabitatDistanceProcessParams } from "../models/HabitatDistanceProcessParams";
import { JobList } from "../models/JobList";
import { LandUseSealedAreaProcessOutputs } from "../models/LandUseSealedAreaProcessOutputs";
import { LandUseSealedAreaProcessParams } from "../models/LandUseSealedAreaProcessParams";
import { LandingPage } from "../models/LandingPage";
import { NDVIProcessOutputs } from "../models/NDVIProcessOutputs";
import { NDVIProcessParams } from "../models/NDVIProcessParams";
import { Process } from "../models/Process";
import { ProcessList } from "../models/ProcessList";
import { Results } from "../models/Results";
import { StatusInfo } from "../models/StatusInfo";
import { UserSession } from "../models/UserSession";

import {
  CapabilitiesApiRequestFactory,
  CapabilitiesApiResponseProcessor,
} from "../apis/CapabilitiesApi";
export class ObservableCapabilitiesApi {
  private requestFactory: CapabilitiesApiRequestFactory;
  private responseProcessor: CapabilitiesApiResponseProcessor;
  private configuration: Configuration;

  public constructor(
    configuration: Configuration,
    requestFactory?: CapabilitiesApiRequestFactory,
    responseProcessor?: CapabilitiesApiResponseProcessor,
  ) {
    this.configuration = configuration;
    this.requestFactory =
      requestFactory || new CapabilitiesApiRequestFactory(configuration);
    this.responseProcessor =
      responseProcessor || new CapabilitiesApiResponseProcessor();
  }

  /**
   * API definition
   */
  public apiWithHttpInfo(
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<{ [key: string]: any }>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.api(_config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.apiWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * API definition
   */
  public api(
    _options?: ConfigurationOptions,
  ): Observable<{ [key: string]: any }> {
    return this.apiWithHttpInfo(_options).pipe(
      map((apiResponse: HttpInfo<{ [key: string]: any }>) => apiResponse.data),
    );
  }

  /**
   * A list of all conformance classes specified in a standard that the server conforms to.
   * API conformance definition
   */
  public conformanceWithHttpInfo(
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<Conformance>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.conformance(_config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.conformanceWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * A list of all conformance classes specified in a standard that the server conforms to.
   * API conformance definition
   */
  public conformance(_options?: ConfigurationOptions): Observable<Conformance> {
    return this.conformanceWithHttpInfo(_options).pipe(
      map((apiResponse: HttpInfo<Conformance>) => apiResponse.data),
    );
  }

  /**
   * The landing page provides links to the API definition and the conformance statements for this API.
   * Landing page
   */
  public rootWithHttpInfo(
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<LandingPage>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.root(_config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.rootWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * The landing page provides links to the API definition and the conformance statements for this API.
   * Landing page
   */
  public root(_options?: ConfigurationOptions): Observable<LandingPage> {
    return this.rootWithHttpInfo(_options).pipe(
      map((apiResponse: HttpInfo<LandingPage>) => apiResponse.data),
    );
  }
}

import {
  DefaultApiRequestFactory,
  DefaultApiResponseProcessor,
} from "../apis/DefaultApi";
export class ObservableDefaultApi {
  private requestFactory: DefaultApiRequestFactory;
  private responseProcessor: DefaultApiResponseProcessor;
  private configuration: Configuration;

  public constructor(
    configuration: Configuration,
    requestFactory?: DefaultApiRequestFactory,
    responseProcessor?: DefaultApiResponseProcessor,
  ) {
    this.configuration = configuration;
    this.requestFactory =
      requestFactory || new DefaultApiRequestFactory(configuration);
    this.responseProcessor =
      responseProcessor || new DefaultApiResponseProcessor();
  }

  /**
   */
  public healthHandlerWithHttpInfo(
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<void>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.healthHandler(_config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.healthHandlerWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   */
  public healthHandler(_options?: ConfigurationOptions): Observable<void> {
    return this.healthHandlerWithHttpInfo(_options).pipe(
      map((apiResponse: HttpInfo<void>) => apiResponse.data),
    );
  }
}

import {
  ProcessesApiRequestFactory,
  ProcessesApiResponseProcessor,
} from "../apis/ProcessesApi";
export class ObservableProcessesApi {
  private requestFactory: ProcessesApiRequestFactory;
  private responseProcessor: ProcessesApiResponseProcessor;
  private configuration: Configuration;

  public constructor(
    configuration: Configuration,
    requestFactory?: ProcessesApiRequestFactory,
    responseProcessor?: ProcessesApiResponseProcessor,
  ) {
    this.configuration = configuration;
    this.requestFactory =
      requestFactory || new ProcessesApiRequestFactory(configuration);
    this.responseProcessor =
      responseProcessor || new ProcessesApiResponseProcessor();
  }

  /**
   * Cancel a job execution and remove it from the jobs list.  For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).
   * Cancel a job execution, remove finished job
   * @param jobId
   */
  public _deleteWithHttpInfo(
    jobId: string,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<StatusInfo>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory._delete(jobId, _config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor._deleteWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * Cancel a job execution and remove it from the jobs list.  For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).
   * Cancel a job execution, remove finished job
   * @param jobId
   */
  public _delete(
    jobId: string,
    _options?: ConfigurationOptions,
  ): Observable<StatusInfo> {
    return this._deleteWithHttpInfo(jobId, _options).pipe(
      map((apiResponse: HttpInfo<StatusInfo>) => apiResponse.data),
    );
  }

  /**
   * @param biodiversitySensitiveAreasProcessParams
   */
  public executeBiodiversitySensitiveAreasWithHttpInfo(
    biodiversitySensitiveAreasProcessParams: BiodiversitySensitiveAreasProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<BiodiversitySensitiveAreasProcessOutputs>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise =
      this.requestFactory.executeBiodiversitySensitiveAreas(
        biodiversitySensitiveAreasProcessParams,
        _config,
      );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.executeBiodiversitySensitiveAreasWithHttpInfo(
                rsp,
              ),
            ),
          );
        }),
      );
  }

  /**
   * @param biodiversitySensitiveAreasProcessParams
   */
  public executeBiodiversitySensitiveAreas(
    biodiversitySensitiveAreasProcessParams: BiodiversitySensitiveAreasProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<BiodiversitySensitiveAreasProcessOutputs> {
    return this.executeBiodiversitySensitiveAreasWithHttpInfo(
      biodiversitySensitiveAreasProcessParams,
      _options,
    ).pipe(
      map(
        (apiResponse: HttpInfo<BiodiversitySensitiveAreasProcessOutputs>) =>
          apiResponse.data,
      ),
    );
  }

  /**
   * @param habitatDistanceProcessParams
   */
  public executeHabitatDistanceWithHttpInfo(
    habitatDistanceProcessParams: HabitatDistanceProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<HabitatDistanceProcessOutputs>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.executeHabitatDistance(
      habitatDistanceProcessParams,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.executeHabitatDistanceWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * @param habitatDistanceProcessParams
   */
  public executeHabitatDistance(
    habitatDistanceProcessParams: HabitatDistanceProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<HabitatDistanceProcessOutputs> {
    return this.executeHabitatDistanceWithHttpInfo(
      habitatDistanceProcessParams,
      _options,
    ).pipe(
      map(
        (apiResponse: HttpInfo<HabitatDistanceProcessOutputs>) =>
          apiResponse.data,
      ),
    );
  }

  /**
   * @param landUseSealedAreaProcessParams
   */
  public executeLandUseSealedAreaWithHttpInfo(
    landUseSealedAreaProcessParams: LandUseSealedAreaProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<LandUseSealedAreaProcessOutputs>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.executeLandUseSealedArea(
      landUseSealedAreaProcessParams,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.executeLandUseSealedAreaWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * @param landUseSealedAreaProcessParams
   */
  public executeLandUseSealedArea(
    landUseSealedAreaProcessParams: LandUseSealedAreaProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<LandUseSealedAreaProcessOutputs> {
    return this.executeLandUseSealedAreaWithHttpInfo(
      landUseSealedAreaProcessParams,
      _options,
    ).pipe(
      map(
        (apiResponse: HttpInfo<LandUseSealedAreaProcessOutputs>) =>
          apiResponse.data,
      ),
    );
  }

  /**
   * @param nDVIProcessParams
   */
  public executeNdviWithHttpInfo(
    nDVIProcessParams: NDVIProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<NDVIProcessOutputs>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.executeNdvi(
      nDVIProcessParams,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.executeNdviWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * @param nDVIProcessParams
   */
  public executeNdvi(
    nDVIProcessParams: NDVIProcessParams,
    _options?: ConfigurationOptions,
  ): Observable<NDVIProcessOutputs> {
    return this.executeNdviWithHttpInfo(nDVIProcessParams, _options).pipe(
      map((apiResponse: HttpInfo<NDVIProcessOutputs>) => apiResponse.data),
    );
  }

  /**
   * Create a new job.  For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).  Schema: <https://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/ogcapi-processes-1.yaml>
   * Execute a process
   * @param processID
   * @param execute
   */
  public executionWithHttpInfo(
    processID: string,
    execute: Execute,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<Results | StatusInfo>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.execution(
      processID,
      execute,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.executionWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * Create a new job.  For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).  Schema: <https://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/ogcapi-processes-1.yaml>
   * Execute a process
   * @param processID
   * @param execute
   */
  public execution(
    processID: string,
    execute: Execute,
    _options?: ConfigurationOptions,
  ): Observable<Results | StatusInfo> {
    return this.executionWithHttpInfo(processID, execute, _options).pipe(
      map((apiResponse: HttpInfo<Results | StatusInfo>) => apiResponse.data),
    );
  }

  /**
   * For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).
   * Retrieve the list of jobs
   * @param [limit] Amount of items to return
   * @param [offset] Offset into the items list
   */
  public jobsWithHttpInfo(
    limit?: number,
    offset?: number,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<JobList>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.jobs(
      limit,
      offset,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.jobsWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).
   * Retrieve the list of jobs
   * @param [limit] Amount of items to return
   * @param [offset] Offset into the items list
   */
  public jobs(
    limit?: number,
    offset?: number,
    _options?: ConfigurationOptions,
  ): Observable<JobList> {
    return this.jobsWithHttpInfo(limit, offset, _options).pipe(
      map((apiResponse: HttpInfo<JobList>) => apiResponse.data),
    );
  }

  /**
   * The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process. That said, the Core requirements class makes the following recommendation:  Implementations SHOULD consider supporting the OGC process description.  For more information, see Section 7.10.
   * Retrieve a processes description
   * @param processID
   */
  public processWithHttpInfo(
    processID: string,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<Process>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.process(
      processID,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.processWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process. That said, the Core requirements class makes the following recommendation:  Implementations SHOULD consider supporting the OGC process description.  For more information, see Section 7.10.
   * Retrieve a processes description
   * @param processID
   */
  public process(
    processID: string,
    _options?: ConfigurationOptions,
  ): Observable<Process> {
    return this.processWithHttpInfo(processID, _options).pipe(
      map((apiResponse: HttpInfo<Process>) => apiResponse.data),
    );
  }

  /**
   * The list of processes contains a summary of each process the OGC API - Processes offers, including the link to a more detailed description of the process.  For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).
   * Retrieve the list of available processes
   */
  public processesWithHttpInfo(
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<ProcessList>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.processes(_config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.processesWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * The list of processes contains a summary of each process the OGC API - Processes offers, including the link to a more detailed description of the process.  For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).
   * Retrieve the list of available processes
   */
  public processes(_options?: ConfigurationOptions): Observable<ProcessList> {
    return this.processesWithHttpInfo(_options).pipe(
      map((apiResponse: HttpInfo<ProcessList>) => apiResponse.data),
    );
  }

  /**
   * Lists available results of a job. In case of a failure, lists exceptions instead.  For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).  
   * Retrieve the result(s) of a job
   * @param jobId
   */
  public resultsWithHttpInfo(
    jobId: string,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<Results>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.results(jobId, _config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.resultsWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * Lists available results of a job. In case of a failure, lists exceptions instead.  For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).  
   * Retrieve the result(s) of a job
   * @param jobId
   */
  public results(
    jobId: string,
    _options?: ConfigurationOptions,
  ): Observable<Results> {
    return this.resultsWithHttpInfo(jobId, _options).pipe(
      map((apiResponse: HttpInfo<Results>) => apiResponse.data),
    );
  }

  /**
   * Shows the status of a job.  For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).
   * Retrieve the status of a job
   * @param jobId
   */
  public statusWithHttpInfo(
    jobId: string,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<StatusInfo>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.status(jobId, _config);
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.statusWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * Shows the status of a job.  For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).
   * Retrieve the status of a job
   * @param jobId
   */
  public status(
    jobId: string,
    _options?: ConfigurationOptions,
  ): Observable<StatusInfo> {
    return this.statusWithHttpInfo(jobId, _options).pipe(
      map((apiResponse: HttpInfo<StatusInfo>) => apiResponse.data),
    );
  }
}

import {
  UserApiRequestFactory,
  UserApiResponseProcessor,
} from "../apis/UserApi";
export class ObservableUserApi {
  private requestFactory: UserApiRequestFactory;
  private responseProcessor: UserApiResponseProcessor;
  private configuration: Configuration;

  public constructor(
    configuration: Configuration,
    requestFactory?: UserApiRequestFactory,
    responseProcessor?: UserApiResponseProcessor,
  ) {
    this.configuration = configuration;
    this.requestFactory =
      requestFactory || new UserApiRequestFactory(configuration);
    this.responseProcessor =
      responseProcessor || new UserApiResponseProcessor();
  }

  /**
   * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
   * @param authCodeResponse
   */
  public authHandlerWithHttpInfo(
    redirectUri: string,
    authCodeResponse: AuthCodeResponse,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<UserSession>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.authHandler(
      redirectUri,
      authCodeResponse,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.authHandlerWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
   * @param authCodeResponse
   */
  public authHandler(
    redirectUri: string,
    authCodeResponse: AuthCodeResponse,
    _options?: ConfigurationOptions,
  ): Observable<UserSession> {
    return this.authHandlerWithHttpInfo(
      redirectUri,
      authCodeResponse,
      _options,
    ).pipe(map((apiResponse: HttpInfo<UserSession>) => apiResponse.data));
  }

  /**
   * Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\'s login page.
   * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
   */
  public authRequestUrlHandlerWithHttpInfo(
    redirectUri: string,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<string>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.authRequestUrlHandler(
      redirectUri,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.authRequestUrlHandlerWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\'s login page.
   * @param redirectUri The URI to which the identity provider should redirect after successful authentication.
   */
  public authRequestUrlHandler(
    redirectUri: string,
    _options?: ConfigurationOptions,
  ): Observable<string> {
    return this.authRequestUrlHandlerWithHttpInfo(redirectUri, _options).pipe(
      map((apiResponse: HttpInfo<string>) => apiResponse.data),
    );
  }

  /**
   * Returns the user\'s credits.
   * @param year
   * @param month
   */
  public getCreditsWithHttpInfo(
    year: number,
    month: number,
    _options?: ConfigurationOptions,
  ): Observable<HttpInfo<GetCreditsResponse>> {
    const _config = mergeConfiguration(this.configuration, _options);

    const requestContextPromise = this.requestFactory.getCredits(
      year,
      month,
      _config,
    );
    // build promise chain
    let middlewarePreObservable = from<RequestContext>(requestContextPromise);
    for (const middleware of _config.middleware) {
      middlewarePreObservable = middlewarePreObservable.pipe(
        mergeMap((ctx: RequestContext) => middleware.pre(ctx)),
      );
    }

    return middlewarePreObservable
      .pipe(mergeMap((ctx: RequestContext) => _config.httpApi.send(ctx)))
      .pipe(
        mergeMap((response: ResponseContext) => {
          let middlewarePostObservable = of(response);
          for (const middleware of _config.middleware.reverse()) {
            middlewarePostObservable = middlewarePostObservable.pipe(
              mergeMap((rsp: ResponseContext) => middleware.post(rsp)),
            );
          }
          return middlewarePostObservable.pipe(
            map((rsp: ResponseContext) =>
              this.responseProcessor.getCreditsWithHttpInfo(rsp),
            ),
          );
        }),
      );
  }

  /**
   * Returns the user\'s credits.
   * @param year
   * @param month
   */
  public getCredits(
    year: number,
    month: number,
    _options?: ConfigurationOptions,
  ): Observable<GetCreditsResponse> {
    return this.getCreditsWithHttpInfo(year, month, _options).pipe(
      map((apiResponse: HttpInfo<GetCreditsResponse>) => apiResponse.data),
    );
  }
}
