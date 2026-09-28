import { HttpInfo } from "../http/http";
import { Configuration, ConfigurationOptions } from "../configuration";

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

import { ObservableCapabilitiesApi } from "./ObservableAPI";
import {
  CapabilitiesApiRequestFactory,
  CapabilitiesApiResponseProcessor,
} from "../apis/CapabilitiesApi";

export interface CapabilitiesApiApiRequest {}

export interface CapabilitiesApiConformanceRequest {}

export interface CapabilitiesApiRootRequest {}

export class ObjectCapabilitiesApi {
  private api: ObservableCapabilitiesApi;

  public constructor(
    configuration: Configuration,
    requestFactory?: CapabilitiesApiRequestFactory,
    responseProcessor?: CapabilitiesApiResponseProcessor,
  ) {
    this.api = new ObservableCapabilitiesApi(
      configuration,
      requestFactory,
      responseProcessor,
    );
  }

  /**
   * API definition
   * @param param the request object
   */
  public apiWithHttpInfo(
    param: CapabilitiesApiApiRequest = {},
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<{ [key: string]: any }>> {
    return this.api.apiWithHttpInfo(options).toPromise();
  }

  /**
   * API definition
   * @param param the request object
   */
  public api_(
    param: CapabilitiesApiApiRequest = {},
    options?: ConfigurationOptions,
  ): Promise<{ [key: string]: any }> {
    return this.api.api(options).toPromise();
  }

  /**
   * A list of all conformance classes specified in a standard that the server conforms to.
   * API conformance definition
   * @param param the request object
   */
  public conformanceWithHttpInfo(
    param: CapabilitiesApiConformanceRequest = {},
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<Conformance>> {
    return this.api.conformanceWithHttpInfo(options).toPromise();
  }

  /**
   * A list of all conformance classes specified in a standard that the server conforms to.
   * API conformance definition
   * @param param the request object
   */
  public conformance(
    param: CapabilitiesApiConformanceRequest = {},
    options?: ConfigurationOptions,
  ): Promise<Conformance> {
    return this.api.conformance(options).toPromise();
  }

  /**
   * The landing page provides links to the API definition and the conformance statements for this API.
   * Landing page
   * @param param the request object
   */
  public rootWithHttpInfo(
    param: CapabilitiesApiRootRequest = {},
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<LandingPage>> {
    return this.api.rootWithHttpInfo(options).toPromise();
  }

  /**
   * The landing page provides links to the API definition and the conformance statements for this API.
   * Landing page
   * @param param the request object
   */
  public root(
    param: CapabilitiesApiRootRequest = {},
    options?: ConfigurationOptions,
  ): Promise<LandingPage> {
    return this.api.root(options).toPromise();
  }
}

import { ObservableDefaultApi } from "./ObservableAPI";
import {
  DefaultApiRequestFactory,
  DefaultApiResponseProcessor,
} from "../apis/DefaultApi";

export interface DefaultApiHealthHandlerRequest {}

export class ObjectDefaultApi {
  private api: ObservableDefaultApi;

  public constructor(
    configuration: Configuration,
    requestFactory?: DefaultApiRequestFactory,
    responseProcessor?: DefaultApiResponseProcessor,
  ) {
    this.api = new ObservableDefaultApi(
      configuration,
      requestFactory,
      responseProcessor,
    );
  }

  /**
   * @param param the request object
   */
  public healthHandlerWithHttpInfo(
    param: DefaultApiHealthHandlerRequest = {},
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<void>> {
    return this.api.healthHandlerWithHttpInfo(options).toPromise();
  }

  /**
   * @param param the request object
   */
  public healthHandler(
    param: DefaultApiHealthHandlerRequest = {},
    options?: ConfigurationOptions,
  ): Promise<void> {
    return this.api.healthHandler(options).toPromise();
  }
}

import { ObservableProcessesApi } from "./ObservableAPI";
import {
  ProcessesApiRequestFactory,
  ProcessesApiResponseProcessor,
} from "../apis/ProcessesApi";

export interface ProcessesApiDeleteRequest {
  /**
   *
   * Defaults to: undefined
   * @type string
   * @memberof ProcessesApi_delete
   */
  jobId: string;
}

export interface ProcessesApiExecuteBiodiversitySensitiveAreasRequest {
  /**
   *
   * @type BiodiversitySensitiveAreasProcessParams
   * @memberof ProcessesApiexecuteBiodiversitySensitiveAreas
   */
  biodiversitySensitiveAreasProcessParams: BiodiversitySensitiveAreasProcessParams;
}

export interface ProcessesApiExecuteHabitatDistanceRequest {
  /**
   *
   * @type HabitatDistanceProcessParams
   * @memberof ProcessesApiexecuteHabitatDistance
   */
  habitatDistanceProcessParams: HabitatDistanceProcessParams;
}

export interface ProcessesApiExecuteLandUseSealedAreaRequest {
  /**
   *
   * @type LandUseSealedAreaProcessParams
   * @memberof ProcessesApiexecuteLandUseSealedArea
   */
  landUseSealedAreaProcessParams: LandUseSealedAreaProcessParams;
}

export interface ProcessesApiExecuteNdviRequest {
  /**
   *
   * @type NDVIProcessParams
   * @memberof ProcessesApiexecuteNdvi
   */
  nDVIProcessParams: NDVIProcessParams;
}

export interface ProcessesApiExecutionRequest {
  /**
   *
   * Defaults to: undefined
   * @type string
   * @memberof ProcessesApiexecution
   */
  processID: string;
  /**
   *
   * @type Execute
   * @memberof ProcessesApiexecution
   */
  execute: Execute;
}

export interface ProcessesApiJobsRequest {
  /**
   * Amount of items to return
   * Minimum: 0
   * Defaults to: undefined
   * @type number
   * @memberof ProcessesApijobs
   */
  limit?: number;
  /**
   * Offset into the items list
   * Minimum: 0
   * Defaults to: undefined
   * @type number
   * @memberof ProcessesApijobs
   */
  offset?: number;
}

export interface ProcessesApiProcessRequest {
  /**
   *
   * Defaults to: undefined
   * @type string
   * @memberof ProcessesApiprocess
   */
  processID: string;
}

export interface ProcessesApiProcessesRequest {}

export interface ProcessesApiResultsRequest {
  /**
   *
   * Defaults to: undefined
   * @type string
   * @memberof ProcessesApiresults
   */
  jobId: string;
}

export interface ProcessesApiStatusRequest {
  /**
   *
   * Defaults to: undefined
   * @type string
   * @memberof ProcessesApistatus
   */
  jobId: string;
}

export class ObjectProcessesApi {
  private api: ObservableProcessesApi;

  public constructor(
    configuration: Configuration,
    requestFactory?: ProcessesApiRequestFactory,
    responseProcessor?: ProcessesApiResponseProcessor,
  ) {
    this.api = new ObservableProcessesApi(
      configuration,
      requestFactory,
      responseProcessor,
    );
  }

  /**
   * Cancel a job execution and remove it from the jobs list.  For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).
   * Cancel a job execution, remove finished job
   * @param param the request object
   */
  public _deleteWithHttpInfo(
    param: ProcessesApiDeleteRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<StatusInfo>> {
    return this.api._deleteWithHttpInfo(param.jobId, options).toPromise();
  }

  /**
   * Cancel a job execution and remove it from the jobs list.  For more information, see [Section 13](https://docs.ogc.org/is/18-062/18-062.html#Dismiss).
   * Cancel a job execution, remove finished job
   * @param param the request object
   */
  public _delete(
    param: ProcessesApiDeleteRequest,
    options?: ConfigurationOptions,
  ): Promise<StatusInfo> {
    return this.api._delete(param.jobId, options).toPromise();
  }

  /**
   * @param param the request object
   */
  public executeBiodiversitySensitiveAreasWithHttpInfo(
    param: ProcessesApiExecuteBiodiversitySensitiveAreasRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<BiodiversitySensitiveAreasProcessOutputs>> {
    return this.api
      .executeBiodiversitySensitiveAreasWithHttpInfo(
        param.biodiversitySensitiveAreasProcessParams,
        options,
      )
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public executeBiodiversitySensitiveAreas(
    param: ProcessesApiExecuteBiodiversitySensitiveAreasRequest,
    options?: ConfigurationOptions,
  ): Promise<BiodiversitySensitiveAreasProcessOutputs> {
    return this.api
      .executeBiodiversitySensitiveAreas(
        param.biodiversitySensitiveAreasProcessParams,
        options,
      )
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public executeHabitatDistanceWithHttpInfo(
    param: ProcessesApiExecuteHabitatDistanceRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<HabitatDistanceProcessOutputs>> {
    return this.api
      .executeHabitatDistanceWithHttpInfo(
        param.habitatDistanceProcessParams,
        options,
      )
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public executeHabitatDistance(
    param: ProcessesApiExecuteHabitatDistanceRequest,
    options?: ConfigurationOptions,
  ): Promise<HabitatDistanceProcessOutputs> {
    return this.api
      .executeHabitatDistance(param.habitatDistanceProcessParams, options)
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public executeLandUseSealedAreaWithHttpInfo(
    param: ProcessesApiExecuteLandUseSealedAreaRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<LandUseSealedAreaProcessOutputs>> {
    return this.api
      .executeLandUseSealedAreaWithHttpInfo(
        param.landUseSealedAreaProcessParams,
        options,
      )
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public executeLandUseSealedArea(
    param: ProcessesApiExecuteLandUseSealedAreaRequest,
    options?: ConfigurationOptions,
  ): Promise<LandUseSealedAreaProcessOutputs> {
    return this.api
      .executeLandUseSealedArea(param.landUseSealedAreaProcessParams, options)
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public executeNdviWithHttpInfo(
    param: ProcessesApiExecuteNdviRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<NDVIProcessOutputs>> {
    return this.api
      .executeNdviWithHttpInfo(param.nDVIProcessParams, options)
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public executeNdvi(
    param: ProcessesApiExecuteNdviRequest,
    options?: ConfigurationOptions,
  ): Promise<NDVIProcessOutputs> {
    return this.api.executeNdvi(param.nDVIProcessParams, options).toPromise();
  }

  /**
   * Create a new job.  For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).  Schema: <https://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/ogcapi-processes-1.yaml>
   * Execute a process
   * @param param the request object
   */
  public executionWithHttpInfo(
    param: ProcessesApiExecutionRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<Results | StatusInfo>> {
    return this.api
      .executionWithHttpInfo(param.processID, param.execute, options)
      .toPromise();
  }

  /**
   * Create a new job.  For more information, see [Section 7.11](https://docs.ogc.org/is/18-062/18-062.html#sc_create_job).  Schema: <https://schemas.opengis.net/ogcapi/processes/part1/1.0/openapi/ogcapi-processes-1.yaml>
   * Execute a process
   * @param param the request object
   */
  public execution(
    param: ProcessesApiExecutionRequest,
    options?: ConfigurationOptions,
  ): Promise<Results | StatusInfo> {
    return this.api
      .execution(param.processID, param.execute, options)
      .toPromise();
  }

  /**
   * For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).
   * Retrieve the list of jobs
   * @param param the request object
   */
  public jobsWithHttpInfo(
    param: ProcessesApiJobsRequest = {},
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<JobList>> {
    return this.api
      .jobsWithHttpInfo(param.limit, param.offset, options)
      .toPromise();
  }

  /**
   * For more information, see [Section 11](https://docs.ogc.org/is/18-062/18-062.html#sc_job_list).
   * Retrieve the list of jobs
   * @param param the request object
   */
  public jobs(
    param: ProcessesApiJobsRequest = {},
    options?: ConfigurationOptions,
  ): Promise<JobList> {
    return this.api.jobs(param.limit, param.offset, options).toPromise();
  }

  /**
   * The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process. That said, the Core requirements class makes the following recommendation:  Implementations SHOULD consider supporting the OGC process description.  For more information, see Section 7.10.
   * Retrieve a processes description
   * @param param the request object
   */
  public processWithHttpInfo(
    param: ProcessesApiProcessRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<Process>> {
    return this.api.processWithHttpInfo(param.processID, options).toPromise();
  }

  /**
   * The process description contains information about inputs and outputs and a link to the execution-endpoint for the process. The Core does not mandate the use of a specific process description to specify the interface of a process. That said, the Core requirements class makes the following recommendation:  Implementations SHOULD consider supporting the OGC process description.  For more information, see Section 7.10.
   * Retrieve a processes description
   * @param param the request object
   */
  public process(
    param: ProcessesApiProcessRequest,
    options?: ConfigurationOptions,
  ): Promise<Process> {
    return this.api.process(param.processID, options).toPromise();
  }

  /**
   * The list of processes contains a summary of each process the OGC API - Processes offers, including the link to a more detailed description of the process.  For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).
   * Retrieve the list of available processes
   * @param param the request object
   */
  public processesWithHttpInfo(
    param: ProcessesApiProcessesRequest = {},
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<ProcessList>> {
    return this.api.processesWithHttpInfo(options).toPromise();
  }

  /**
   * The list of processes contains a summary of each process the OGC API - Processes offers, including the link to a more detailed description of the process.  For more information, see [Section 7.9](https://docs.ogc.org/is/18-062/18-062.html#sc_process_list).
   * Retrieve the list of available processes
   * @param param the request object
   */
  public processes(
    param: ProcessesApiProcessesRequest = {},
    options?: ConfigurationOptions,
  ): Promise<ProcessList> {
    return this.api.processes(options).toPromise();
  }

  /**
   * Lists available results of a job. In case of a failure, lists exceptions instead.  For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).  
   * Retrieve the result(s) of a job
   * @param param the request object
   */
  public resultsWithHttpInfo(
    param: ProcessesApiResultsRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<Results>> {
    return this.api.resultsWithHttpInfo(param.jobId, options).toPromise();
  }

  /**
   * Lists available results of a job. In case of a failure, lists exceptions instead.  For more information, see [Section 7.13](https://docs.ogc.org/is/18-062r2/18-062r2.html#sc_retrieve_job_results).  
   * Retrieve the result(s) of a job
   * @param param the request object
   */
  public results(
    param: ProcessesApiResultsRequest,
    options?: ConfigurationOptions,
  ): Promise<Results> {
    return this.api.results(param.jobId, options).toPromise();
  }

  /**
   * Shows the status of a job.  For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).
   * Retrieve the status of a job
   * @param param the request object
   */
  public statusWithHttpInfo(
    param: ProcessesApiStatusRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<StatusInfo>> {
    return this.api.statusWithHttpInfo(param.jobId, options).toPromise();
  }

  /**
   * Shows the status of a job.  For more information, see [Section 7.12](https://docs.ogc.org/is/18-062/18-062.html#sc_retrieve_status_info).
   * Retrieve the status of a job
   * @param param the request object
   */
  public status(
    param: ProcessesApiStatusRequest,
    options?: ConfigurationOptions,
  ): Promise<StatusInfo> {
    return this.api.status(param.jobId, options).toPromise();
  }
}

import { ObservableUserApi } from "./ObservableAPI";
import {
  UserApiRequestFactory,
  UserApiResponseProcessor,
} from "../apis/UserApi";

export interface UserApiAuthHandlerRequest {
  /**
   * The URI to which the identity provider should redirect after successful authentication.
   * Defaults to: undefined
   * @type string
   * @memberof UserApiauthHandler
   */
  redirectUri: string;
  /**
   *
   * @type AuthCodeResponse
   * @memberof UserApiauthHandler
   */
  authCodeResponse: AuthCodeResponse;
}

export interface UserApiAuthRequestUrlHandlerRequest {
  /**
   * The URI to which the identity provider should redirect after successful authentication.
   * Defaults to: undefined
   * @type string
   * @memberof UserApiauthRequestUrlHandler
   */
  redirectUri: string;
}

export interface UserApiGetCreditsRequest {
  /**
   *
   * Minimum: 0
   * Defaults to: undefined
   * @type number
   * @memberof UserApigetCredits
   */
  year: number;
  /**
   *
   * Minimum: 0
   * Defaults to: undefined
   * @type number
   * @memberof UserApigetCredits
   */
  month: number;
}

export class ObjectUserApi {
  private api: ObservableUserApi;

  public constructor(
    configuration: Configuration,
    requestFactory?: UserApiRequestFactory,
    responseProcessor?: UserApiResponseProcessor,
  ) {
    this.api = new ObservableUserApi(
      configuration,
      requestFactory,
      responseProcessor,
    );
  }

  /**
   * @param param the request object
   */
  public authHandlerWithHttpInfo(
    param: UserApiAuthHandlerRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<UserSession>> {
    return this.api
      .authHandlerWithHttpInfo(
        param.redirectUri,
        param.authCodeResponse,
        options,
      )
      .toPromise();
  }

  /**
   * @param param the request object
   */
  public authHandler(
    param: UserApiAuthHandlerRequest,
    options?: ConfigurationOptions,
  ): Promise<UserSession> {
    return this.api
      .authHandler(param.redirectUri, param.authCodeResponse, options)
      .toPromise();
  }

  /**
   * Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\'s login page.
   * @param param the request object
   */
  public authRequestUrlHandlerWithHttpInfo(
    param: UserApiAuthRequestUrlHandlerRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<string>> {
    return this.api
      .authRequestUrlHandlerWithHttpInfo(param.redirectUri, options)
      .toPromise();
  }

  /**
   * Generates a URL for initiating the OIDC code flow, which the frontend can use to redirect the user to the identity provider\'s login page.
   * @param param the request object
   */
  public authRequestUrlHandler(
    param: UserApiAuthRequestUrlHandlerRequest,
    options?: ConfigurationOptions,
  ): Promise<string> {
    return this.api
      .authRequestUrlHandler(param.redirectUri, options)
      .toPromise();
  }

  /**
   * Returns the user\'s credits.
   * @param param the request object
   */
  public getCreditsWithHttpInfo(
    param: UserApiGetCreditsRequest,
    options?: ConfigurationOptions,
  ): Promise<HttpInfo<GetCreditsResponse>> {
    return this.api
      .getCreditsWithHttpInfo(param.year, param.month, options)
      .toPromise();
  }

  /**
   * Returns the user\'s credits.
   * @param param the request object
   */
  public getCredits(
    param: UserApiGetCreditsRequest,
    options?: ConfigurationOptions,
  ): Promise<GetCreditsResponse> {
    return this.api.getCredits(param.year, param.month, options).toPromise();
  }
}
